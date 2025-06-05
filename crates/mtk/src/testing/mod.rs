use std::path::PathBuf;

use crate::{RepoPathBuf, FsEntry};
use crate::file_tree::FileType;

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
