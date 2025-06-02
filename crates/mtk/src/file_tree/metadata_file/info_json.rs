use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

pub fn find_associated_paths(info_path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let info_dir = info_path.parent().expect("info.json file has no parent");
    let info_filename = info_path
        .file_name()
        .unwrap()
        .to_str()
        .expect("info.json file has no filename");

    let mut associated_paths = Vec::new();
    if let Some(base_filename) = info_filename.strip_suffix(".info.json") {
        for entry in std::fs::read_dir(info_dir)? {
            let entry_path = entry?.path();
            if entry_path.is_file() {
                let entry_filename = entry_path.file_name().unwrap().to_str().unwrap();
                if entry_filename.starts_with(base_filename) && entry_filename != info_filename {
                    associated_paths.push(entry_path);
                }
            }
        }
    }

    Ok(associated_paths)
}

// Check if this is an InfoJsonFile
pub fn is_info_json(path: &Path) -> bool {
    path
        .file_name()
        .and_then(|f| f.to_str())
        .is_some_and(|f| f.ends_with(".info.json"))
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedInfoJson {
    pub linked_urls: Vec<String>,
}

impl ParsedInfoJson {
    fn from_raw_info(raw_info: Value) -> ParsedInfoJson {
        let webpage_url = &raw_info["webpage_url"];
        let linked_urls = match webpage_url {
            Value::String(url) => {
                vec![url.to_owned()]
            }
            _ => Vec::new(),
        };

        ParsedInfoJson { linked_urls }
    }

    #[cfg(test)]
    fn from_str(info_json_str: &str) -> ParsedInfoJson {
        ParsedInfoJson::from_raw_info(serde_json::from_str(info_json_str).unwrap())
    }

    pub fn from_file(info_json_path: &Path) -> std::io::Result<ParsedInfoJson> {
        let info_json_file = File::open(info_json_path)?;
        let reader = BufReader::new(info_json_file);
        let raw_info: Value = serde_json::from_reader(reader)
            .unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", info_json_path, e));
        Ok(ParsedInfoJson::from_raw_info(raw_info))
    }
}

pub fn get_associated_info(info_path: &Path) -> std::io::Result<HashMap<PathBuf, serde_json::Value>> {
    let associated_paths = find_associated_paths(info_path)?;
    let info_value = serde_json::to_value(ParsedInfoJson::from_file(info_path)?)?;
    let mut associated_info = HashMap::new();
    for ap in associated_paths {
        associated_info.insert(ap.clone(), info_value.clone());
    }
    Ok(associated_info)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::testing::testdata_path;

    use super::*;

    fn touch(path: &Path) -> std::io::Result<()> {
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)?;
        Ok(())
    }

    #[test]
    fn test_is_info_json() {
        assert!(is_info_json(Path::new("file.info.json")));
        assert!(!is_info_json(Path::new("file.json")));
        assert!(!is_info_json(Path::new("file.info")));
        assert!(!is_info_json(Path::new("file.mp4")));
    }

    #[test]
    fn test_nothing() {
        let parsed = ParsedInfoJson::from_str(
            r#"{
            "video_ext": "mp4"
        }"#,
        );
        assert_eq!(parsed.linked_urls.len(), 0);
    }

    #[test]
    fn test_webpage_url() {
        let parsed = ParsedInfoJson::from_str(
            r#"{
            "title": "abcdef",
            "webpage_url": "http://www.example.com/video/abcdef"
        }"#,
        );
        assert_eq!(parsed.linked_urls.len(), 1);
        assert_eq!(parsed.linked_urls[0], "http://www.example.com/video/abcdef");
    }

    #[test]
    fn test_find_associated_paths() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;

        // Create file fixtures
        touch(&temp_dir.path().join("file1.mkv"))?;
        touch(&temp_dir.path().join("file1.mp4"))?;
        touch(&temp_dir.path().join("file1.info.json"))?;
        touch(&temp_dir.path().join("file2.info.json"))?;

        let ap1 = find_associated_paths(&temp_dir.path().join("file1.info.json"))?;
        assert!(ap1.contains(&temp_dir.path().join("file1.mkv")));
        assert!(ap1.contains(&temp_dir.path().join("file1.mp4")));
        assert_eq!(ap1.len(), 2);

        let ap2 = find_associated_paths(&temp_dir.path().join("file2.info.json"))?;
        assert!(ap2.is_empty());

        Ok(())
    }

    #[test]
    fn test_get_associated_info() -> std::io::Result<()> {
        let root = testdata_path("metadata_file");

        fn get_value<OutT>(value: &serde_json::Value, key: &str) -> OutT
        where
            OutT: serde::de::DeserializeOwned,
        {
            serde_json::from_value::<OutT>(
                value.get(key)
                    .expect(&format!("no such key {}", key))
                    .clone()
            ).expect("deserialize error")
        }

        let ai1 = get_associated_info(&root.join("file1.info.json"))?;
        assert_eq!(
            get_value::<Vec<String>>(
                ai1.get(&root.join("file1.txt")).expect("file1.txt"),
                "linked_urls"
            ),
            ["http://example.com"],
        );
        assert_eq!(
            get_value::<Vec<String>>(
                ai1.get(&root.join("file1.mp4")).expect("file1.mp4"),
                "linked_urls"
            ),
            ["http://example.com"],
        );
        assert!(!ai1.contains_key(&root.join("file1.info.json")));

        let ai2 = get_associated_info(&root.join("orphan.info.json"))?;
        assert!(ai2.is_empty());

        Ok(())
    }
}
