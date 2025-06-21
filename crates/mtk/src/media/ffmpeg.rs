use std::io::BufRead;

/// Convert ffprobe's "codec_name" to "fancy" format
/// Test with:
/// https://cconcolato.github.io/media-mime-support/
pub fn codec_name_to_fancy(codec_name: &str) -> &str {
    match codec_name {
        // https://stackoverflow.com/questions/47371381/how-do-i-specify-the-hevc-codec-in-the-html5-video-source-type-attribute
        "hevc" => "hvc1",
        // https://jakearchibald.com/2022/html-codecs-parameter-for-av1/
        "av1" => "av01.0.08M.10",
        _ => codec_name,
    }
}

/// Convert ffprobe's "codec_name" to RFC 6381 format
pub fn codec_name_to_rfc6381(codec_name: &str) -> &str {
    match codec_name {
        "h264" => "avc1",
        "hevc" => codec_name_to_fancy(codec_name),
        "vp9" => "vp09",
        "vp8" => "vp08",
        "av1" => codec_name_to_fancy(codec_name),
        _ => codec_name,
    }
}

/// Check the result of a Command, print the stdout/stderr if it failed
pub fn check_command_output(output: std::process::Output, msg: &str) {
    if !output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("{}: {}", msg, output.status);
    }
}

struct ProgressParser {
    time_secs: f32,
    done: bool,
}

impl ProgressParser {
    fn parse_line(&mut self, line: &str) -> bool {
        // Parse one key=value line
        if let Some((key, value)) = line.split_once('=') {
            // Does this line have more = signs? If so, it's the human-readable
            // progress line, and we shouldn't process it.
            if value.contains('=') {
                return false;
            }

            // Is this the "progress" line?
            if key == "progress" {
                self.done = true;
                return true;
            }
            // Is this one of the keys we recognize?
            else if key == "out_time_us" {
                if value.trim() == "N/A" {
                    return false;
                } else if let Ok(time_us) = value.parse::<f32>() {
                    self.time_secs = time_us / 1_000_000.0;
                } else {
                    println!("Failed to parse out_time_us: {}", value);
                    return false;
                }
            }
        }

        // If none of the above, we don't care about this line
        return false;
    }

    fn get(&mut self) -> f32 {
        assert!(self.done);
        self.done = false;
        self.time_secs
    }
}

/// Run ffmpeg command and get progress updates
pub fn ffmpeg_progress_updates(
    cmd: &mut std::process::Command,
    mut progress_cb: impl FnMut(f32),
) {
    let mut child = cmd.arg("-progress").arg("pipe:1")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn ffmpeg");

    let raw_stdout = child.stdout.take().expect("stdout");
    let stdout = std::io::BufReader::new(raw_stdout);
    let mut progress_parser = ProgressParser { time_secs: 0.0, done: false };
    for line in stdout.lines() {
        let line = line.expect("read line");
        let complete = progress_parser.parse_line(&line);
        if complete {
            progress_cb(progress_parser.get());
        }
    }
}
