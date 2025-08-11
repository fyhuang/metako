use std::path::Path;

use serde::Serialize;

use mtk::{file_tree::GeneratedFile, Entry};
use mtk::media::video;

use super::file_renderer::{RawFileRenderer, GeneratedFileRenderer, ServableFileRenderer};

fn get_type_with_codecs(video_info: &video::VideoInfo) -> String {
    if video_info.mime_type == "video/mp4" {
        format!("video/mp4; codecs=\"{}\"", video_info.codec_rfc6381)
    } else {
        format!("{}; codecs=\"{}\"", video_info.mime_type, video_info.codec)
    }
}

#[derive(Clone, Serialize)]
pub struct VideoSourceRenderer {
    pub file: ServableFileRenderer,
    pub type_with_codecs: String,
}

impl VideoSourceRenderer {
    pub fn from_entry(le: &Entry, video_info: &video::VideoInfo) -> VideoSourceRenderer {
        VideoSourceRenderer {
            file: ServableFileRenderer::RawFile(RawFileRenderer {
                repo_path: le.fs.repo_path.0.clone(),
            }),
            type_with_codecs: get_type_with_codecs(&video_info),
        }
    }

    pub fn from_generated(gfile: &GeneratedFile, file_path: &Path) -> VideoSourceRenderer {
        let video_info = video::get_video_info(file_path).expect("Failed to get video info");
        VideoSourceRenderer {
            file: ServableFileRenderer::GeneratedFile(GeneratedFileRenderer::new(gfile)),
            type_with_codecs: get_type_with_codecs(&video_info),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct VideoPlayerRenderer {
    pub main_source: VideoSourceRenderer,
    pub alt_formats: Vec<VideoSourceRenderer>,

    pub loop_and_autoplay: bool,
}

impl VideoPlayerRenderer {
    pub fn new(vault: &mtk::Vault, entry: &Entry) -> VideoPlayerRenderer {
        let gen_tree = vault.new_generated_tree();
        let video_info = video::get_video_info(&entry.fs.file_path).expect("get_video_info");

        let mut alt_formats = Vec::new();
        for alt_format_gfile in gen_tree
            .query_generated_files(entry.db.id, mtk::file_tree::GeneratedFileType::AltFormat)
        {
            let path = gen_tree.path_to_generated_file(&alt_format_gfile);
            alt_formats.push(VideoSourceRenderer::from_generated(
                &alt_format_gfile,
                &path,
            ));
        }

        VideoPlayerRenderer {
            main_source: VideoSourceRenderer::from_entry(entry, &video_info),
            alt_formats: alt_formats,

            loop_and_autoplay: video_info.duration_secs <= 180.,
        }
    }
}
