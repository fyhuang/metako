use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use crate::vault::META_DIRNAME;
use crate::CatalogError;
use crate::catalog::Catalog;
use crate::vault::FilerConfig;
use crate::file_tree::{FileTree, GeneratedTree};
use crate::userdata::HistoryDb;

fn same_mount_point(p1: &Path, p2: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let p1_meta = std::fs::metadata(p1).expect("metadata p1");
    let p2_meta = std::fs::metadata(p2).expect("metadata p2");
    p1_meta.dev() == p2_meta.dev()
}

fn check_meta_dir(meta_dir: &Path) {
    assert!(meta_dir.file_name().unwrap().to_str().unwrap() == META_DIRNAME);
    assert!(meta_dir.is_dir());
}

pub struct Vault {
    pub file_root: PathBuf,
    pub meta_root: PathBuf,
    pub config: FilerConfig,
}

impl Vault {
    pub fn from_meta_dir(meta_dir: &Path) -> Vault {
        check_meta_dir(meta_dir);

        let config_path = meta_dir.join(super::config::CONFIG_FILENAME);

        let config_or = FilerConfig::load_from(&config_path);
        let config_file_root = config_or.as_ref().map(|c| c.file_root.as_ref()).flatten();
        let file_root = if let Some(file_root) = &config_file_root {
            file_root
        } else {
            let parent = meta_dir.parent().unwrap();
            // TODO(fyhuang): check that meta dir is named correctly for default file root
            println!("Default file root: {}", parent.display());
            parent
        }.to_path_buf();
        
        Vault {
            file_root: file_root.clone(),
            meta_root: meta_dir.to_path_buf(),
            config: config_or.unwrap_or(FilerConfig {
                file_root: Some(file_root.clone()),
                skip_paths: Vec::new(),
                include_non_media: false,
                default_save_parent: BTreeMap::new(),
                local_path_prefixes: Vec::new(),
            }),
        }
    }
    
    pub fn from_data_dir(data_dir: &Path) -> Vault {
        // Find a .mtk directory in a parent of the data directory
        let mut parent = data_dir.to_path_buf();
        loop {
            if !same_mount_point(&parent, &data_dir) {
                panic!("No {} directory found; not walking across mount points", META_DIRNAME);
            }

            let meta_dir = parent.join(META_DIRNAME);
            if meta_dir.is_dir() {
                return Self::from_meta_dir(&meta_dir);
            }
            if !parent.pop() {
                panic!("No {} directory found in parent directories", META_DIRNAME);
            }
        }
    }

    pub fn from_cwd() -> Vault {
        let cwd = std::env::current_dir().unwrap();
        Self::from_data_dir(&cwd)
    }

    pub fn entries_json_path(&self) -> PathBuf {
        self.meta_root.join("entries.json")
    }

    pub fn new_file_tree(&self) -> FileTree {
        let skip_paths = self.config.skip_paths.iter().map(|s| self.file_root.join(s)).collect();
        FileTree::new(&self.file_root, skip_paths, self.config.include_non_media)
    }

    pub fn new_generated_tree(&self) -> GeneratedTree {
        GeneratedTree::new(&self.meta_root)
    }

    pub fn open_catalog(&self) -> Result<Catalog, CatalogError> {
        let db_path = self.meta_root.join("catalog.db");
        //println!("Catalog at path {}", db_path.display());
        Catalog::open(&db_path)
    }

    pub fn open_history_db(&self) -> HistoryDb {
        let db_path = self.meta_root.join("history.db");
        println!("HistoryDb at path {}", db_path.display());
        HistoryDb::new(&db_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::testing;

    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_from_meta_dir_empty() {
        let temp_dir = tempdir().unwrap();
        let meta_dir = temp_dir.path().join(META_DIRNAME);
        std::fs::create_dir(&meta_dir).unwrap();

        let stash = Vault::from_meta_dir(&meta_dir);

        assert_eq!(stash.file_root, temp_dir.path());
        assert_eq!(stash.meta_root, meta_dir);
        assert!(stash.config.file_root.is_some());

        // Make sure we can create objects
        let _ = stash.new_file_tree();
        let _ = stash.open_catalog();
    }

    #[test]
    fn test_from_meta_dir_with_config() {
        let temp_dir = tempdir().unwrap();
        let meta_dir = temp_dir.path().join(META_DIRNAME);
        std::fs::create_dir(&meta_dir).unwrap();

        let config_path = meta_dir.join(super::super::config::CONFIG_FILENAME);
        std::fs::File::create(&config_path).unwrap();

        let stash = Vault::from_meta_dir(&meta_dir);

        assert_eq!(stash.file_root, temp_dir.path());
        assert_eq!(stash.meta_root, meta_dir);
        assert!(stash.config.file_root.is_some());

        // Make sure we can create objects
        let _ = stash.new_file_tree();
        let _ = stash.open_catalog();
    }

    #[test]
    fn test_from_data_dir() {
        let testdata_path = testing::testdata_path("mixed");
        let stash = Vault::from_data_dir(&testdata_path);

        // Make sure we can create objects
        let _ = stash.new_file_tree();
        let _ = stash.open_catalog();
    }
}
