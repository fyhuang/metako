use std::path::Path;

use crate::preview;
use crate::file_tree::{GeneratedFile, GeneratedFileType};

#[allow(unused)]
fn minivid_gfile(entry_id: i64) -> GeneratedFile {
    GeneratedFile {
        entry_id: entry_id,
        file_type: GeneratedFileType::Minivid,
        metadata: "".to_string(),
        extension: "webm".to_string(),
    }
}

fn pick_preview_timestamp(duration_secs: f64) -> f64 {
    // Pick a timestamp to grab a frame from

    // Scale the frame to pick based on the duration of the video:
    //  - 60m and over: 15% of the total duration
    //  - 10m: 35% of the total duration
    //  - 1m and under: 50% of the total duration
    // The shorter the video, the more likely that the interesting part is toward the middle.
    let frame_timestamp_secs = if duration_secs >= 60.0 * 60.0 {
        duration_secs * 0.15
    } else if duration_secs <= 60.0 {
        duration_secs * 0.5
    } else {
        // Piecewise linear interpolation
        let interp = |x1, x2, y1, y2, x| {
            let slope = (y2 - y1) / (x2 - x1);
            let intercept = y1 - slope * x1;
            slope * x + intercept
        };
        if duration_secs >= 60.0 * 10.0 {
            duration_secs * interp(60.0 * 60.0, 60.0 * 10.0, 0.15, 0.35, duration_secs)
        } else {
            duration_secs * interp(60.0 * 10.0, 60.0, 0.35, 0.5, duration_secs)
        }
    };
    println!("duration: {}, frame_timestamp_secs: {}", duration_secs, frame_timestamp_secs);

    frame_timestamp_secs
}

pub fn make_preview_image(video_path: &Path, out_path: &Path) {
    // Get total duration of video
    let output = std::process::Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(video_path)
        .output()
        .expect("failed to execute ffprobe");
    let duration_secs_result = std::str::from_utf8(&output.stdout).expect("from_utf8").trim().parse::<f64>();
    if duration_secs_result.is_err() {
        // TODO: actually return the errro
        eprintln!("Failed to get duration of video: {:?}", video_path);
        return;
    }
    let duration_secs = duration_secs_result.expect("parse");
    let frame_timestamp_secs = pick_preview_timestamp(duration_secs);

    // Pick the frame at the time and save it to disk
    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.arg("-ss")
        .arg(format!("{}s", frame_timestamp_secs))
        .arg("-i")
        .arg(video_path)
        .arg("-vframes")
        .arg("1");
    preview::ffmpeg_preview_args(&mut cmd);
    cmd.arg(out_path)
        .output()
        .expect("failed to execute ffmpeg");
}

#[allow(unused)]
fn make_minivid(video_path: &Path, out_path: &Path) {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testing;

    // TODO: combine with the one in image.rs
    fn get_image_dimensions(image_path: &Path) -> (u32, u32) {
        // Use ffprobe to get the dimensions of the image
        let output = std::process::Command::new("ffprobe")
            .arg("-v")
            .arg("error")
            .arg("-select_streams")
            .arg("v:0")
            .arg("-show_entries")
            .arg("stream=width,height")
            .arg("-of")
            .arg("default=noprint_wrappers=1:nokey=1")
            .arg(image_path)
            .output()
            .expect("failed to execute ffprobe");

        let stdout = std::str::from_utf8(&output.stdout).expect("from_utf8");
        let mut lines = stdout.lines();
        let width = lines.next().expect("width").parse().expect("parse");
        let height = lines.next().expect("height").parse().expect("parse");
        (width, height)
    }

    #[test]
    fn test_pick_preview_timestamp() {
        // Boundary cases
        assert_eq!(pick_preview_timestamp(60.0), 30.0);
        assert_eq!(pick_preview_timestamp(600.0), 210.0);
        assert_eq!(pick_preview_timestamp(3600.0), 540.0);

        // Interpolated cases
        assert!((pick_preview_timestamp(330.0) - 140.25).abs() < 0.01);
        assert!((pick_preview_timestamp(2100.0) - 525.0).abs() < 0.01);
    }

    #[test]
    fn test_make_preview_image() {
        let root = testing::testdata_path("preview");
        let dest_file = tempfile::Builder::new()
            .suffix(".jpg")
            .tempfile()
            .expect("temp file");
        std::fs::remove_file(dest_file.path()).expect("remove temp file");

        make_preview_image(&root.join("short_video.mp4"), dest_file.path());
        assert!(dest_file.path().exists());
        assert_eq!(get_image_dimensions(dest_file.path()), (320, 240));
    }
}
