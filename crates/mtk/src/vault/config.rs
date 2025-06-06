use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use serde::Deserialize;

pub const CONFIG_FILENAME: &str = "config.json";

// TODO(fyhuang): make config generic for plugins
#[derive(Debug, Deserialize)]
pub struct FilerConfig {
    pub file_root: Option<PathBuf>,
    // TODO(fyhuang): use a ignore file instead of hardcoded paths
    #[serde(default)]
    pub skip_paths: Vec<String>,
    #[serde(default)]
    pub include_non_media: bool,
    #[serde(default)]
    pub default_save_parent: BTreeMap<String, String>,
    #[serde(default)]
    pub local_path_prefixes: Vec<String>,
}

impl FilerConfig {
    pub fn load_from(filepath: &Path) -> Option<FilerConfig> {
        let data = std::fs::read(filepath).ok()?;
        let data_str = std::str::from_utf8(&data).ok()?;
        if data_str.is_empty() {
            return None;
        }

        serde_json::from_str(data_str).expect(&format!("couldn't load config from {:?}", filepath))
    }
}
