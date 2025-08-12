use std::path::Path;

use crate::file_tree::GeneratedFile;

use super::VideoInfo;
use crate::media::ffmpeg;

#[derive(Copy, Clone, Debug)]
pub enum TranscodeProfile {
    H264_200K,

    AV1_200K,
    AV1_400K,
}

impl TranscodeProfile {
    pub fn from_str(profile_str: &str) -> Option<TranscodeProfile> {
        match profile_str {
            "h264_200k" => Some(TranscodeProfile::H264_200K),
            "av1_200k" => Some(TranscodeProfile::AV1_200K),
            "av1_400k" => Some(TranscodeProfile::AV1_400K),
            _ => None,
        }
    }

    pub fn to_gen_file(&self, entry_id: i64) -> GeneratedFile {
        let metadata = match self {
            TranscodeProfile::H264_200K => "h264_200k".to_string(),
            TranscodeProfile::AV1_200K => "av1_200k".to_string(),
            TranscodeProfile::AV1_400K => "av1_400k".to_string(),
        };
        let extension = match self {
            TranscodeProfile::H264_200K => "mp4".to_string(),
            TranscodeProfile::AV1_200K => "webm".to_string(),
            TranscodeProfile::AV1_400K => "webm".to_string(),
        };
        GeneratedFile {
            entry_id,
            file_type: crate::file_tree::GeneratedFileType::AltFormat,
            metadata,
            extension,
        }
    }

    fn two_pass(&self) -> bool {
        match self {
            TranscodeProfile::H264_200K => false,
            TranscodeProfile::AV1_200K => false, // SVT-AV1 doesn't have 2pass?
            TranscodeProfile::AV1_400K => false, // SVT-AV1 doesn't have 2pass?
        }
    }

    fn transcode_args_video(&self, cmd: &mut std::process::Command) {
        match self {
            TranscodeProfile::H264_200K => {
                // Scale to 360p, keeping aspect ratio
                // TODO: prevent scaling up
                cmd.arg("-vf").arg("scale=-4:360");
                cmd.arg("-c:v").arg("libx264");
                cmd.arg("-preset").arg("slower");
                cmd.arg("-b:v").arg("200k");
            }
            // See SVT-AV1 docs for recommendations:
            // https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/master/Docs/Ffmpeg.md#example-2-encoding-for-personal-use
            TranscodeProfile::AV1_200K => {
                // Scale to 480p, keeping aspect ratio
                // TODO: prevent scaling up
                cmd.arg("-vf").arg("scale=-4:480");
                cmd.arg("-pix_fmt").arg("yuv420p10le"); // 10-bit improves encode quality
                cmd.arg("-c:v").arg("libsvtav1");
                cmd.arg("-b:v").arg("200k");
                cmd.arg("-preset").arg("5");
                cmd.arg("-svtav1-params").arg("film-grain=8");
                cmd.arg("-g").arg("300");
            }
            TranscodeProfile::AV1_400K => {
                // Scale to 720p, keeping aspect ratio
                // TODO: prevent scaling up
                cmd.arg("-vf").arg("scale=-4:720");
                cmd.arg("-pix_fmt").arg("yuv420p10le"); // 10-bit improves encode quality
                cmd.arg("-c:v").arg("libsvtav1");
                cmd.arg("-b:v").arg("400k");
                cmd.arg("-preset").arg("5");
                cmd.arg("-svtav1-params").arg("film-grain=4");
                cmd.arg("-g").arg("300");
            }
        }
    }

    fn transcode_args_audio(&self, cmd: &mut std::process::Command) {
        match self {
            TranscodeProfile::H264_200K => {
                cmd.arg("-c:a").arg("aac");
                cmd.arg("-b:a").arg("32k");
            }
            TranscodeProfile::AV1_200K => {
                cmd.arg("-c:a").arg("libopus");
                cmd.arg("-b:a").arg("32k");
            }
            TranscodeProfile::AV1_400K => {
                cmd.arg("-c:a").arg("libopus");
                cmd.arg("-b:a").arg("48k");
            }
        }
    }
}

pub fn transcode_alt_format(video_path: &Path, out_path: &Path, profile: TranscodeProfile) {
    if profile.two_pass() {
        let two_pass_log = out_path.with_extension("log");

        // Pass 1
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.arg("-i")
            .arg(video_path);
        profile.transcode_args_video(&mut cmd);
        cmd.arg("-an")
            .arg("-pass").arg("1")
            .arg("-passlogfile").arg(&two_pass_log)
            .arg("-f").arg("null")
            .arg("/dev/null");

        ffmpeg::ffmpeg_progress_updates(&mut cmd, |time_secs| {
            println!("Pass 1: {:.1} seconds", time_secs);
        });

        // Pass 2
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.arg("-i")
            .arg(video_path);
        profile.transcode_args_video(&mut cmd);
        profile.transcode_args_audio(&mut cmd);
        cmd.arg("-pass").arg("2")
            .arg("-passlogfile").arg(&two_pass_log);
        cmd.arg(out_path);

        ffmpeg::ffmpeg_progress_updates(&mut cmd, |time_secs| {
            println!("Pass 2: {:.1} seconds", time_secs);
        });

        // Remove the log file(s)
        glob::glob(&format!("{}.*", two_pass_log.to_str().unwrap()))
            .expect("glob")
            .for_each(|entry| {
                let path = entry.expect("entry");
                std::fs::remove_file(&path).expect("remove two-pass log");
            });
    } else {
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.arg("-i")
            .arg(video_path);
        profile.transcode_args_video(&mut cmd);
        profile.transcode_args_audio(&mut cmd);
        // TODO: should we ignore errors?
        cmd.arg("-err_detect").arg("ignore_err");
        cmd.arg(out_path);
        ffmpeg::ffmpeg_progress_updates(&mut cmd, |time_secs| {
            println!("Transcoding progress: {:.1} seconds", time_secs);
        });
    }
}

/// Predict whether Chrome can play a given video.
/// If not, the video is a candidate for transcoding.
pub fn chrome_can_play(video_info: &VideoInfo) -> bool {
    let mime_type = video_info.mime_type.as_str();
    let codec = video_info.codec.as_str();
    let codec_rfc6381 = video_info.codec_rfc6381.as_str();

    if mime_type == "video/mp4" {
        match codec_rfc6381 {
            "avc1" => true,
            // Chrome can't consistently play HEVC yet
            "hvc1" => false,
            "vp09" => true,
            "vp08" => true,
            "av01" => true,
            _ => false,
        }
    } else if mime_type == "video/webm" {
        match codec {
            "h264" => true,
            // Neither Safari nor Chrome can play HEVC in mkv
            // https://www.reddit.com/r/jellyfin/comments/yennxi/hevc_in_chromechromium/
            "hevc" => false,
            "vp9" => true,
            "vp8" => true,
            "av1" => true,
            _ => false,
        }
    } else {
        false
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::testing;

    #[test]
    #[ignore]
    fn test_transcode_h264() -> Result<(), Box<dyn std::error::Error>> {
        let file_root = testing::testdata_path("transcode");
        let tempdir = tempfile::tempdir().expect("tempdir");

        let starting_file_count = std::fs::read_dir(&file_root)
            .expect("read_dir")
            .count();
        let video_path = file_root.join("vidaud_h265_aac.mkv");

        transcode_alt_format(&video_path, &tempdir.path().join("out.mp4"), TranscodeProfile::H264_200K);

        assert!(tempdir.path().join("out.mp4").exists(), "Output file should exist");

        // Make sure there aren't any temp files left over
        let after_file_count = std::fs::read_dir(&file_root)
            .expect("read_dir")
            .count();
        assert_eq!(starting_file_count, after_file_count, "No extra files should be created");

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_transcode_av1() -> Result<(), Box<dyn std::error::Error>> {
        let file_root = testing::testdata_path("transcode");
        let tempdir = tempfile::tempdir().expect("tempdir");

        let starting_file_count = std::fs::read_dir(&file_root)
            .expect("read_dir")
            .count();
        let video_path = file_root.join("vidaud_h265_aac.mkv");

        for profile in [
            TranscodeProfile::AV1_200K,
            TranscodeProfile::AV1_400K,
        ] {
            let out_path = tempdir.path().join(format!("out_{}.webm", profile.to_gen_file(0).metadata));
            transcode_alt_format(&video_path, &out_path, profile);

            assert!(out_path.exists(), "Output file should exist for {:?}", profile);
        }

        // Make sure there aren't any temp files left over
        let after_file_count = std::fs::read_dir(&file_root)
            .expect("read_dir")
            .count();
        assert_eq!(starting_file_count, after_file_count, "No extra files should be created");

        Ok(())
    }

    #[test]
    fn test_chrome_can_play() {
        let video_info = VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "h264".to_string(),
            codec_rfc6381: "avc1".to_string(),
            duration_secs: 120.0,
            width: 1920,
            height: 1080,
            bitrate: 5000000,
        };
        assert!(chrome_can_play(&video_info));

        let video_info = VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "hevc".to_string(),
            codec_rfc6381: "hvc1".to_string(),
            duration_secs: 120.0,
            width: 1920,
            height: 1080,
            bitrate: 5000000,
        };
        assert!(!chrome_can_play(&video_info));
    }
}
