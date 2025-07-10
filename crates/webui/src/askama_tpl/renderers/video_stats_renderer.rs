use mtk::{catalog::generated_notes, Entry};
use mtk::media::video;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct VideoStatsRenderer {
    pub duration_str: String,
    pub vertical: bool,
    // One of "sd", "hd", "4k", "8k"
    pub resolution_label: String,
    // Is this a VR video? (180, fisheye, or 360; or 3D SBS including non-surround)
    pub is_vr: bool,
}

impl VideoStatsRenderer {
    pub fn from(entry: &Entry) -> VideoStatsRenderer {
        match generated_notes::read::<video::VideoInfo>(&entry.db, video::VIDEO_INFO_GROUP_NAME) {
            Some(video_info) => VideoStatsRenderer::from_info(video_info),
            // No info available; return a default renderer
            None => VideoStatsRenderer {
                duration_str: "--:--".to_string(),
                vertical: false,
                resolution_label: "??".to_string(),
                is_vr: false,
            }
        }
    }

    fn from_info(video_info: video::VideoInfo) -> VideoStatsRenderer {
        let hours = (video_info.duration_secs / 3600.).floor();
        let minutes = ((video_info.duration_secs - (hours * 3600.)) / 60.).floor();
        let seconds = video_info.duration_secs - (hours * 3600.) - (minutes * 60.);

        let min_dimension = std::cmp::min(video_info.width, video_info.height);
        let resolution_label = if min_dimension >= 4320 {
            "8k"
        } else if min_dimension >= 2160 {
            "4k"
        } else if min_dimension >= 720 {
            "hd"
        } else {
            "sd"
        };

        VideoStatsRenderer {
            duration_str: if hours.floor() > 0. {
                format!("{:.0}:{:02.0}:{:02.0}", hours, minutes, seconds.floor())
            } else {
                format!("{:02.0}:{:02.0}", minutes, seconds.floor())
            },
            vertical: video_info.height > video_info.width,
            resolution_label: resolution_label.to_string(),
            is_vr: false, // TODO: implement VR detection
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_stats_duration_small() {
        let r = VideoStatsRenderer::from_info(video::VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "".to_string(),
            codec_rfc6381: "".to_string(),
            duration_secs: 123.45,
            width: 0,
            height: 0,
            bitrate: 0,
        });
        assert_eq!(r.duration_str, "02:03");
    }

    #[test]
    fn test_video_stats_duration_large() {
        let r = VideoStatsRenderer::from_info(video::VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "".to_string(),
            codec_rfc6381: "".to_string(),
            duration_secs: 12345.67,
            width: 0,
            height: 0,
            bitrate: 0,
        });
        assert_eq!(r.duration_str, "3:25:45");
    }

    #[test]
    fn test_video_stats_vertical() {
        assert_eq!(VideoStatsRenderer::from_info(video::VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "".to_string(),
            codec_rfc6381: "".to_string(),
            duration_secs: 0.,
            width: 1280,
            height: 720,
            bitrate: 0,
        }).vertical, false);

        // Square is not considered vertical
        assert_eq!(VideoStatsRenderer::from_info(video::VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "".to_string(),
            codec_rfc6381: "".to_string(),
            duration_secs: 0.,
            width: 512,
            height: 512,
            bitrate: 0,
        }).vertical, false);

        assert_eq!(VideoStatsRenderer::from_info(video::VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "".to_string(),
            codec_rfc6381: "".to_string(),
            duration_secs: 0.,
            width: 1080,
            height: 1920,
            bitrate: 0,
        }).vertical, true);
    }

    #[test]
    fn test_video_stats_resolution() {
        assert_eq!(VideoStatsRenderer::from_info(video::VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "".to_string(),
            codec_rfc6381: "".to_string(),
            duration_secs: 0.,
            width: 640,
            height: 480,
            bitrate: 0,
        }).resolution_label, "sd");

        // Aspect ratio doesn't matter
        assert_eq!(VideoStatsRenderer::from_info(video::VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "".to_string(),
            codec_rfc6381: "".to_string(),
            duration_secs: 0.,
            width: 480,
            height: 640,
            bitrate: 0,
        }).resolution_label, "sd");

        // Both 720p and 1080p considered "hd"
        assert_eq!(VideoStatsRenderer::from_info(video::VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "".to_string(),
            codec_rfc6381: "".to_string(),
            duration_secs: 0.,
            width: 1280,
            height: 720,
            bitrate: 0,
        }).resolution_label, "hd");

        assert_eq!(VideoStatsRenderer::from_info(video::VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "".to_string(),
            codec_rfc6381: "".to_string(),
            duration_secs: 0.,
            width: 1920,
            height: 1080,
            bitrate: 0,
        }).resolution_label, "hd");

        // 4k
        assert_eq!(VideoStatsRenderer::from_info(video::VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "".to_string(),
            codec_rfc6381: "".to_string(),
            duration_secs: 0.,
            width: 3840,
            height: 2160,
            bitrate: 0,
        }).resolution_label, "4k");

        // 8k
        assert_eq!(VideoStatsRenderer::from_info(video::VideoInfo {
            mime_type: "video/mp4".to_string(),
            codec: "".to_string(),
            codec_rfc6381: "".to_string(),
            duration_secs: 0.,
            width: 7680,
            height: 4320,
            bitrate: 0,
        }).resolution_label, "8k");
    }
}
