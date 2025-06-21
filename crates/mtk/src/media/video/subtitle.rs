use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use serde::Deserialize;

use crate::file_tree::{GeneratedFileType, GeneratedTree};

#[derive(Debug, Clone, PartialEq)]
pub enum SubtitleSource {
    WebVTTFile(PathBuf),
    OtherFile(PathBuf),
    Embedded { video_path: PathBuf, stream: i32 },
}

#[derive(Debug, Clone)]
pub struct Subtitle {
    pub source: SubtitleSource,
    pub lang_country: String,
}

fn find_subtitle_files_same_dir(video_path: &Path) -> HashMap<String, Subtitle> {
    // Look for subtitle files in the same directory, following the Facebook format.
    // https://www.facebook.com/help/1528795707381162
    let parent_dir = video_path.parent().expect("parent directory");
    let parent_dir_str = parent_dir.to_str().expect("parent directory to_str");
    let mut video_prefix = video_path
        .file_stem()
        .expect("file stem")
        .to_str()
        .expect("file stem to_str")
        .to_string();
    // Add the "." so it gets stripped off by strip_prefix
    video_prefix.push('.');

    fn find_files_internal<F>(
        result: &mut HashMap<String, Subtitle>,
        parent_dir_str: &str,
        video_prefix: &str,
        extension_suffix: &str,
        constructor: F,
    ) where
        F: Fn(PathBuf) -> SubtitleSource,
    {
        // "extension_suffix" should be something like ".srt"
        let entries = glob::glob(&format!("{}/*{}", parent_dir_str, extension_suffix)).unwrap();

        for entry in entries {
            let path = entry.unwrap();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if let Some(without_video_name) = file_name.strip_prefix(&video_prefix) {
                let lang_country = without_video_name
                    .strip_suffix(extension_suffix)
                    .expect("strip_suffix")
                    .to_string();
                result.insert(
                    lang_country.clone(),
                    Subtitle {
                        source: (constructor)(path),
                        lang_country,
                    },
                );
            }
        }
    }

    let mut result = HashMap::new();
    find_files_internal(
        &mut result,
        parent_dir_str,
        &video_prefix,
        ".srt",
        SubtitleSource::OtherFile,
    );
    find_files_internal(
        &mut result,
        parent_dir_str,
        &video_prefix,
        ".vtt",
        SubtitleSource::WebVTTFile,
    );

    result
}

fn find_embedded_subtitles(video_path: &Path) -> HashMap<String, Subtitle> {
    #[derive(Deserialize)]
    struct FfprobeStream {
        index: i32,
        codec_type: String,
        tags: Option<FfprobeTags>,
    }

    #[derive(Deserialize)]
    struct FfprobeTags {
        language: Option<String>,
    }

    #[derive(Deserialize)]
    struct FfprobeOutput {
        streams: Vec<FfprobeStream>,
    }

    // Using ffprobe, find subtitle streams in the video
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-print_format")
        .arg("json")
        .arg("-show_streams")
        .arg(video_path)
        .output()
        .expect("failed to execute ffprobe");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ffprobe_output: FfprobeOutput =
        serde_json::from_str(&stdout).expect("failed to parse ffprobe output");

    let mut subtitles = HashMap::new();
    for stream in ffprobe_output.streams {
        if stream.codec_type == "subtitle" {
            let lang_country = stream
                .tags
                .and_then(|tags| tags.language)
                .unwrap_or("unknown".to_string());
            subtitles.insert(
                lang_country.clone(),
                Subtitle {
                    source: SubtitleSource::Embedded {
                        video_path: video_path.to_path_buf(),
                        stream: stream.index,
                    },
                    lang_country,
                },
            );
        }
    }

    subtitles
}

fn find_generated_subtitles(gen_tree: &GeneratedTree, entry_id: i64) -> HashMap<String, Subtitle> {
    let mut result = HashMap::new();
    for gfile in gen_tree.query_generated_files(entry_id, GeneratedFileType::Subtitle) {
        let lang_country = gfile.metadata.clone();
        result.insert(
            lang_country.clone(),
            Subtitle {
                source: SubtitleSource::WebVTTFile(gen_tree.path_to_generated_file(&gfile)),
                lang_country,
            },
        );
    }
    result
}

pub fn find_non_vtt_subtitles(video_path: &Path) -> HashMap<String, Subtitle> {
    // Find subtitle files (and embedded streams) that have not been converted to WebVTT
    let mut subtitles = find_embedded_subtitles(video_path);
    subtitles.extend(find_subtitle_files_same_dir(video_path));

    // Remove subtitles that are already in WebVTT format
    subtitles.retain(|_, subtitle| match &subtitle.source {
        SubtitleSource::WebVTTFile(_) => false,
        _ => true,
    });

    subtitles
}

pub fn convert_to_vtt(source: &Subtitle, dest_vtt_path: &Path) {
    let mut command = Command::new("ffmpeg");

    // Setup the input arguments
    match &source.source {
        SubtitleSource::WebVTTFile(path) => {
            // This is already in WebVTT format, so this function was probably called in error
            panic!("convert_one_subtitle called with WebVTT source: {:?}", path);
        }
        SubtitleSource::OtherFile(path) => {
            command.arg("-i").arg(path);
        }
        SubtitleSource::Embedded { video_path, stream } => {
            // Select the subtitle stream by index
            // See: <https://trac.ffmpeg.org/wiki/Map>
            command
                .arg("-i")
                .arg(video_path)
                .arg("-map")
                .arg(&format!("0:{}", stream));
        }
    };

    let output = command
        .arg("-c:s")
        .arg("webvtt")
        .arg(dest_vtt_path)
        .output()
        .expect("failed to spawn ffmpeg");
    if !output.status.success() {
        println!("args: {:?}", command);
        println!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        println!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("ffmpeg failed: {:?}", output.status);
    }
}

pub fn find_all_vtt_subtitles(
    video_path: &Path,
    gen_tree: &GeneratedTree,
    entry_id: i64,
) -> Vec<Subtitle> {
    let mut subtitles = find_subtitle_files_same_dir(video_path);
    subtitles.extend(find_generated_subtitles(gen_tree, entry_id));
    subtitles.retain(|_, subtitle| match &subtitle.source {
        SubtitleSource::WebVTTFile(_) => true,
        _ => false,
    });
    subtitles.values().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::testdata_path;

    #[test]
    fn test_find_subtitle_files() {
        let root = testdata_path("subtitle");
        let video_path = root.join("art001m1203451716~small_10s.mp4");
        let files = find_subtitle_files_same_dir(&video_path);
        assert_eq!(files.len(), 2);

        let zh_cn_subtitle = files.get("zh_CN").expect("zh_CN subtitle");
        assert_eq!(
            zh_cn_subtitle.source,
            SubtitleSource::OtherFile(root.join("art001m1203451716~small_10s.zh_CN.srt"))
        );

        let fr_fr_subtitle = files.get("fr_FR").expect("fr_FR subtitle");
        assert_eq!(
            fr_fr_subtitle.source,
            SubtitleSource::WebVTTFile(root.join("art001m1203451716~small_10s.fr_FR.vtt"))
        );
    }

    #[test]
    fn test_find_embedded_subtitles() {
        let root = testdata_path("subtitle");
        let video_path = root.join("art001m1203451716~small_10s.mp4");
        let subtitles = find_embedded_subtitles(&video_path);
        assert_eq!(subtitles.len(), 2);

        let eng_subtitle = subtitles.get("eng").expect("eng subtitle");
        assert_eq!(
            eng_subtitle.source,
            SubtitleSource::Embedded {
                video_path: video_path.clone(),
                stream: 2
            }
        );

        let jpn_subtitle = subtitles.get("jpn").expect("jpn subtitle");
        assert_eq!(
            jpn_subtitle.source,
            SubtitleSource::Embedded {
                video_path: video_path.clone(),
                stream: 3
            }
        );
    }

    #[test]
    fn test_convert_to_vtt_other_file() {
        let root = testdata_path("subtitle");
        let source_path = root.join("art001m1203451716~small_10s.zh_CN.srt");

        let dest_file = tempfile::Builder::new()
            .suffix(".vtt")
            .tempfile()
            .expect("temp file");
        std::fs::remove_file(dest_file.path()).expect("remove temp file");
        let dest_vtt_path = dest_file.path();

        let subtitle = Subtitle {
            source: SubtitleSource::OtherFile(source_path),
            lang_country: "zh_CN".to_string(),
        };

        convert_to_vtt(&subtitle, &dest_vtt_path);

        assert!(dest_vtt_path.exists());
    }

    #[test]
    fn test_convert_to_vtt_embedded() {
        let root = testdata_path("subtitle");
        let video_path = root.join("art001m1203451716~small_10s.mp4");

        let dest_file = tempfile::Builder::new()
            .suffix(".vtt")
            .tempfile()
            .expect("temp file");
        std::fs::remove_file(dest_file.path()).expect("remove temp file");
        let dest_vtt_path = dest_file.path();

        let subtitle = Subtitle {
            source: SubtitleSource::Embedded {
                video_path: video_path.clone(),
                stream: 2,
            },
            lang_country: "eng".to_string(),
        };

        convert_to_vtt(&subtitle, &dest_vtt_path);

        assert!(dest_vtt_path.exists());
    }
}
