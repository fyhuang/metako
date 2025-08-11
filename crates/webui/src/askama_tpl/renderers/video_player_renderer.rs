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
pub struct SubtitleRenderer {
    pub file: ServableFileRenderer,
    pub srclang: String,
}

#[derive(Clone, Serialize)]
pub struct VideoPlayerRenderer {
    pub main_source: VideoSourceRenderer,
    pub alt_formats: Vec<VideoSourceRenderer>,
    pub vtt_subtitles: Vec<SubtitleRenderer>,

    pub loop_and_autoplay: bool,
}

impl VideoPlayerRenderer {
    pub fn new(vault: &mtk::Vault, entry: &Entry) -> VideoPlayerRenderer {
        let file_tree = vault.new_file_tree();
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

        let mut vtt_subtitles = Vec::new();
        for subtitle in video::find_all_vtt_subtitles(&entry.fs.file_path, &gen_tree, entry.db.id) {
            if let video::SubtitleSource::WebVTTFile(path) = &subtitle.source {
                if gen_tree.is_generated(&path) {
                    vtt_subtitles.push(SubtitleRenderer {
                        // TODO: better if we could just return GeneratedFile from video::find_all_vtt_subtitles
                        file: ServableFileRenderer::GeneratedFile(GeneratedFileRenderer {
                            entry_id: entry.db.id,
                            file_type_2l: mtk::file_tree::GeneratedFileType::Subtitle
                                .to_two_letter_code()
                                .to_string(),
                            metadata: subtitle.lang_country.clone(),
                            extension: "vtt".to_string(),
                        }),
                        srclang: subtitle.lang_country.clone(),
                    });
                } else {
                    let repo_path = file_tree
                        .full_to_repo_path(&path)
                        .expect("full_to_repo_path");
                    vtt_subtitles.push(SubtitleRenderer {
                        file: ServableFileRenderer::RawFile(RawFileRenderer {
                            repo_path: repo_path.to_string(),
                        }),
                        srclang: subtitle.lang_country.clone(),
                    })
                }
            } else {
                panic!("Only WebVTTFile can be rendered, not {:?}", subtitle);
            }
        }

        VideoPlayerRenderer {
            main_source: VideoSourceRenderer::from_entry(entry, &video_info),
            alt_formats: alt_formats,
            vtt_subtitles: vtt_subtitles,

            loop_and_autoplay: video_info.duration_secs <= 180.,
        }
    }
}
