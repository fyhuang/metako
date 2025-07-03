use serde::Serialize;

use mtk::Entry;
use mtk::media::video;

use super::file_renderer::RawFileRenderer;

fn get_type_with_codecs(video_info: &video::VideoInfo) -> String {
    if video_info.mime_type == "video/mp4" {
        format!("video/mp4; codecs=\"{}\"", video_info.codec_rfc6381)
    } else {
        format!("{}; codecs=\"{}\"", video_info.mime_type, video_info.codec)
    }
}

#[derive(Clone, Serialize)]
pub struct VideoSourceRenderer {
    pub file: RawFileRenderer,
    pub type_with_codecs: String,
}

impl VideoSourceRenderer {
    pub fn from_entry(le: &Entry, video_info: &video::VideoInfo) -> VideoSourceRenderer {
        VideoSourceRenderer {
            file: RawFileRenderer {
                repo_path: le.fs.repo_path.0.clone(),
            },
            type_with_codecs: get_type_with_codecs(&video_info),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct VideoPlayerRenderer {
    pub main_source: VideoSourceRenderer,

    pub loop_and_autoplay: bool,
}

impl VideoPlayerRenderer {
    pub fn new(entry: &Entry) -> VideoPlayerRenderer {
        let video_info = video::get_video_info(&entry.fs.file_path).expect("get_video_info");
        VideoPlayerRenderer {
            main_source: VideoSourceRenderer::from_entry(entry, &video_info),

            loop_and_autoplay: video_info.duration_secs <= 180.,
        }
    }
}
