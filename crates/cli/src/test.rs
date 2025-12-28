use std::path::PathBuf;

use clap::Parser;

use mtk::media::{image, video};
use mtk::save;

#[derive(Parser)]
pub enum TestSubcommand {
    GetVideoInfo {
        path: PathBuf,
    },
    Preview {
        in_path: PathBuf,
        out_path: PathBuf,
    },
    AltFormat {
        in_path: PathBuf,
        out_path: PathBuf,
        profile: String,
    },
    DownloadYtDlp {
        url: String,
        dest_dir: PathBuf,
    },
    DownloadGalleryDl {
        url: String,
        dest_dir: PathBuf,
    },
    AnalyzeUrl {
        url: String,
    },
}

impl TestSubcommand {
    pub fn run(&self) {
        match self {
            TestSubcommand::GetVideoInfo { path } => {
                let format = video::get_video_info(path);
                println!("{:?}", format);
            }
            TestSubcommand::Preview { in_path, out_path } => {
                if mtk::filetype::is_image(in_path) {
                    image::make_preview_image(&in_path, &out_path);
                } else {
                    video::video_preview::make_preview_image(&in_path, &out_path);
                }
            }
            TestSubcommand::AltFormat {
                in_path,
                out_path,
                profile: profile_str,
            } => {
                let profile = video::transcode::TranscodeProfile::from_str(profile_str)
                    .expect("profile");
                video::transcode::transcode_alt_format(&in_path, &out_path, profile);
            }
            TestSubcommand::DownloadYtDlp { url, dest_dir } => {
                println!("Downloading with yt-dlp: {}", url);
                println!("Destination: {}", dest_dir.display());

                let target = save::SaveTarget::YtDlp {
                    url: url.clone(),
                };

                save::download(&target, dest_dir).expect("download");
            }
            TestSubcommand::DownloadGalleryDl { url, dest_dir } => {
                println!("Downloading with gallery-dl: {}", url);
                println!("Destination: {}", dest_dir.display());

                let target = save::SaveTarget::GalleryDl {
                    url: url.clone(),
                };

                save::download(&target, dest_dir).expect("download");
            }
            TestSubcommand::AnalyzeUrl { url } => {
                println!("Analyzing URL: {}", url);

                let parsed_url = url::Url::parse(url).expect("Invalid URL");
                let result = save::analyze(&parsed_url).expect("analyze");

                println!("{}", serde_json::to_string_pretty(&result).expect("to_string_pretty"));
            }
        }
    }
}
