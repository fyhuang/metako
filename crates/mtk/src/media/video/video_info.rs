use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::media::ffmpeg;

pub const VIDEO_INFO_GROUP_NAME: &'static str = "video";

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    pub mime_type: String,
    pub codec: String,
    pub codec_rfc6381: String,
    pub duration_secs: f64,
    pub width: u32,
    pub height: u32,
    pub bitrate: u64,
}

/// Extract a VideoInfo from a video file using ffprobe.
///
/// This is slow, prefer using the cached version in Entry::generated_notes if
/// many are needed (e.g. during dir listings).
pub fn get_video_info(video_path: &Path) -> Result<VideoInfo, Box<dyn std::error::Error>> {
    // Call ffprobe to get:
    // - mime type
    // - codec name
    // - duration
    // - resolution
    // - bitrate
    #[derive(Deserialize)]
    struct FfprobeStream {
        codec_name: String,
        width: u32,
        height: u32,
    }

    #[derive(Deserialize)]
    struct FfprobeFormat {
        duration: String,
        bit_rate: String,
    }

    #[derive(Deserialize)]
    struct FfprobeOutput {
        streams: Vec<FfprobeStream>,
        format: FfprobeFormat,
    }

    let output = std::process::Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-show_entries")
        .arg("stream=codec_name,width,height")
        .arg("-show_entries")
        .arg("format=duration,bit_rate")
        .arg("-of")
        .arg("json")
        .arg(video_path)
        .output()
        .expect("ffprobe");

    let ffprobe_output: FfprobeOutput =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;
    if ffprobe_output.streams.len() != 1 {
        return Err(
            format!(
                "Expected exactly one video stream, got {:?}",
                ffprobe_output.streams.len()
            )
            .into(),
        );
    }

    let mime_type = match video_path
        .extension()
        .and_then(|ext| ext.to_str())
        .expect("ext.to_str")
    {
        "mp4" => "video/mp4",
        "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/webm",
        "avi" => "video/x-msvideo",
        "wmv" => "video/x-ms-wmv",
        "mov" => "video/quicktime",
        _ => panic!("Unknown video extension {:?}", video_path.extension()),
    };

    Ok(VideoInfo {
        mime_type: mime_type.to_string(),
        codec: ffmpeg::codec_name_to_fancy(&ffprobe_output.streams[0].codec_name)
            .to_string(),
        codec_rfc6381: ffmpeg::codec_name_to_rfc6381(&ffprobe_output.streams[0].codec_name)
            .to_string(),
        duration_secs: ffprobe_output.format.duration.parse()?,
        width: ffprobe_output.streams[0].width,
        height: ffprobe_output.streams[0].height,
        bitrate: ffprobe_output.format.bit_rate.parse()?,
    })
}
