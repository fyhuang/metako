use std::path::PathBuf;

use serde::Serialize;

use crate::RepoPathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct FileType {
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool, // Do we need this?
}

impl FileType {
    pub fn from_metadata(metadata: &std::fs::Metadata) -> FileType {
        FileType {
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            is_symlink: metadata.file_type().is_symlink(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FsEntry {
    pub repo_path: RepoPathBuf,
    pub file_path: PathBuf,
    pub file_name: String,

    // Derived from std::fs::Metadata
    pub file_type: FileType,
    pub size_bytes: u64,
    pub mod_time: chrono::DateTime<chrono::Utc>,

    // If true, this file is a "metadata file" that contains external info for other files.
    pub is_metadata_file: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_filetype_from_metadata_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("testfile.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "hello").unwrap();

        let metadata = fs::metadata(&file_path).unwrap();
        let ft = FileType::from_metadata(&metadata);
        assert!(ft.is_file);
        assert!(!ft.is_dir);
        // Symlink is false for regular file
        assert!(!ft.is_symlink);
    }

    #[test]
    fn test_filetype_from_metadata_dir() {
        let dir = tempdir().unwrap();

        let metadata = fs::metadata(dir.path()).unwrap();
        let ft = FileType::from_metadata(&metadata);
        assert!(!ft.is_file);
        assert!(ft.is_dir);
        assert!(!ft.is_symlink);
    }

    #[test]
    fn test_filetype_from_metadata_symlink() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("target.txt");
        File::create(&file_path).unwrap();

        let symlink_path = dir.path().join("link.txt");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&file_path, &symlink_path).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&file_path, &symlink_path).unwrap();

        let metadata = fs::symlink_metadata(&symlink_path).unwrap();
        let ft = FileType::from_metadata(&metadata);
        // TODO: would be useful to know if the symlink points to a file or directory
        assert!(!ft.is_file); // symlink itself is not a file
        assert!(!ft.is_dir);
        assert!(ft.is_symlink);
    }
}
