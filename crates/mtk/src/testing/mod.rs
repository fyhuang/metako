use std::path::{Path, PathBuf};

use crate::{RepoPathBuf, Entry, FsEntry};
use crate::file_tree::{FileType, FileTree};

pub fn test_fs_entry(repo_path_str: &str) -> FsEntry {
    let repo_path = RepoPathBuf::from(repo_path_str);
    let file_name = repo_path.file_name().to_string();
    FsEntry {
        repo_path: repo_path,
        file_path: PathBuf::from(repo_path_str),
        file_name: file_name,
        file_type: FileType {
            // TODO(fyhuang): make this configurable
            is_file: true,
            is_dir: false,
            is_symlink: false,
        },
        size_bytes: 42,
        mod_time: chrono::DateTime::from_timestamp(0, 0).expect("from_timestamp"),
        is_metadata_file: false,
    }
}


pub fn fake_entry(repo_path_str: &str) -> Entry {
    Entry {
        fs: test_fs_entry(repo_path_str),
        db: crate::catalog::db_entry::DbEntry::default(i64::MIN, RepoPathBuf::from(repo_path_str)),
    }
}

pub fn entry_for(repo_path_str: &str, file_tree: &FileTree, catalog: &mut crate::catalog::Catalog) -> Result<Entry, Box<dyn std::error::Error>> {
    let fs_entry = file_tree.get_fs_entry(&RepoPathBuf::from(repo_path_str))?;
    let db_entry = catalog.get_or_create(&fs_entry);
    Ok(Entry {
        fs: fs_entry,
        db: db_entry,
    })
}


/// Return the path to one of the testdata folders
pub fn testdata_path(name: &str) -> PathBuf {
    // Path to "base" crate
    let base_crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let testdata_path = base_crate_path // metako/crates/mtk
        .parent().unwrap() // metako/crates
        .parent().unwrap() // metako
        .join("testdata").join(name);
    assert!(testdata_path.is_dir());
    testdata_path
}

/// Create a new Vault in a tempdir
pub fn tempdir_vault(file_root: &Path) -> std::io::Result<(tempfile::TempDir, crate::Vault)> {
    let tempdir = tempfile::tempdir()?;
    let meta_path = tempdir.path().join(crate::vault::META_DIRNAME);
    std::fs::create_dir(&meta_path)?;

    // Write a config file that points to the intended file root
    std::fs::write(meta_path.join("config.json"), format!(r##"
        {{"file_root": "{}"}}
    "##, file_root.display()))?;

    let vault = crate::Vault::from_meta_dir(&meta_path);
    Ok((tempdir, vault))
}

/// Create an in-memory sqlite connection
pub fn in_memory_conn(name: &str) -> rusqlite::Connection {
    if name.is_empty() {
        rusqlite::Connection::open_in_memory().expect("Failed to open in-memory database")
    } else {
        // Open a shared connection with the name provided
        let path = format!("file:{}?mode=memory&cache=shared", name);
        rusqlite::Connection::open(&path).unwrap()
    }
}

// From <https://www.bluxte.net/musings/2023/01/08/improving_failure_messages_rust_tests/>
#[derive(Debug)]
pub enum TestError {}

impl<T: std::fmt::Display> From<T> for TestError {
    #[track_caller] // Will show the location of the caller in test failure messages
    fn from(error: T) -> Self {
        panic!("error: {} - {}", std::any::type_name::<T>(), error);
    }
}

pub type TestResult = Result<(), TestError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testdata_path() {
        let path = testdata_path("mixed");
        println!("{:?}", path);
        assert!(path.is_dir());
    }
}
