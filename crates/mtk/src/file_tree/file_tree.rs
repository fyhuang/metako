use std::path::{Path, PathBuf};
use std::collections::VecDeque;

use crate::{filetype, RepoPathBuf};

use super::{FsEntry, FileType};
use super::metadata_file::info_json;

const EADIR_NAME: &str = "@eaDir";

fn mod_time_from_metadata(metadata: &std::fs::Metadata) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::<chrono::Utc>::from(metadata.modified().unwrap())
}

fn is_metadata_file(file_path: &Path) -> bool {
    info_json::is_info_json(file_path)
}

pub type AssociatedInfo = std::collections::HashMap<RepoPathBuf, serde_json::Value>;

impl FsEntry {
    pub fn from_path(path: &Path, base_path: &Path) -> std::io::Result<FsEntry> {
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let metadata = std::fs::metadata(path)?;

        Ok(FsEntry {
            repo_path: RepoPathBuf::from_full_path(base_path, path).unwrap(),
            file_path: path.to_path_buf(),
            file_name: file_name,

            file_type: FileType::from_metadata(&metadata),
            size_bytes: metadata.len(),
            mod_time: mod_time_from_metadata(&metadata),

            is_metadata_file: is_metadata_file(path),
        })
    }

    pub fn from_dir_entry(dir_entry: &std::fs::DirEntry, base_path: &Path) -> std::io::Result<FsEntry> {
        let file_path = dir_entry.path();
        // TODO(fyhuang): generate it directly from dir_entry to save a system call
        Self::from_path(&file_path, base_path)
    }
}

pub struct ListDirRecurIterator {
    file_tree: FileTree,

    read_dir: Option<std::fs::ReadDir>,
    paths: VecDeque<PathBuf>,

    recurse: bool,
}

impl ListDirRecurIterator {
    fn next_entry(&mut self) -> std::io::Result<Option<std::fs::DirEntry>> {
        loop {
            if self.read_dir.is_none() {
                let next_path = self.paths.pop_front();
                if next_path.is_none() {
                    return Ok(None);
                }
                self.read_dir = Some(std::fs::read_dir(&next_path.unwrap())?);
            }

            let dir_entry = self.read_dir.as_mut().unwrap().next();
            if let Some(dir_entry) = dir_entry {
                let dir_entry = dir_entry?;
                let is_dir = dir_entry.file_type()?.is_dir();

                if self.file_tree.should_skip_path(&dir_entry.path(), is_dir) {
                    continue;
                }

                // If we're recursing, add to paths queue if it's a dir
                if self.recurse {
                    if is_dir {
                        self.paths.push_back(dir_entry.path());
                    }
                }
                return Ok(Some(dir_entry));
            } else {
                self.read_dir = None;
            }
        }
    }
}

impl Iterator for ListDirRecurIterator {
    type Item = FsEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let dir_entry = self.next_entry().unwrap()?;
        Some(FsEntry::from_dir_entry(&dir_entry, &self.file_tree.base_path).unwrap())
    }
}


#[derive(Clone)]
pub struct FileTree {
    base_path: PathBuf,

    // TODO(fyhuang): just pass a Config object?
    skip_paths: Vec<PathBuf>,
    include_non_media: bool,
}

impl FileTree {
    fn should_skip_path(&self, file_path: &Path, is_dir: bool) -> bool {
        let file_name = file_path
            .file_name().expect("should_skip_path needs file_name")
            .to_str().expect("filenames should be UTF-8");

        // TODO(fyhuang): make this configurable
        if file_name.starts_with('.') {
            return true;
        }
        if file_name == EADIR_NAME {
            return true;
        }

        if !self.include_non_media {
            let is_media = filetype::is_video(file_path) ||
                filetype::is_image(file_path) ||
                filetype::is_document(file_path);

            if !is_dir && !is_media {
                // Only exception is for metadata files
                if !is_metadata_file(file_path) {
                    return true;
                }
            }
        }

        let file_path_for_contains = file_path.to_path_buf();
        if self.skip_paths.contains(&file_path_for_contains) {
            return true;
        }

        return false;
    }

    // TODO(fyhuang): can we avoid making this pub?
    pub fn repo_to_full_path(&self, repo_path: &RepoPathBuf) -> PathBuf {
        repo_path.to_full_path(&self.base_path)
    }

    pub fn full_to_repo_path(&self, file_path: &Path) -> Option<RepoPathBuf> {
        let abs_file_path = super::abs_path::to_abs_path(file_path);
        RepoPathBuf::from_full_path(&self.base_path, &abs_file_path)
    }

    pub fn new(base_path: &Path, skip_paths: Vec<PathBuf>, include_non_media: bool) -> FileTree {
        FileTree {
            base_path: base_path.canonicalize().unwrap(),
            skip_paths: skip_paths,
            include_non_media: include_non_media,
        }
    }

    pub fn open_read(&self, repo_path: &RepoPathBuf) -> std::io::Result<std::fs::File> {
        let full_path = self.repo_to_full_path(repo_path);
        std::fs::File::open(&full_path)
    }

    pub fn get_fs_entry(&self, repo_path: &RepoPathBuf) -> std::io::Result<FsEntry> {
        let full_path = self.repo_to_full_path(repo_path);
        FsEntry::from_path(&full_path, &self.base_path)
    }

    // If this is a metadata file, get associated files and parsed metadata.
    pub fn read_metadata_file(&self, repo_path: &RepoPathBuf) -> std::io::Result<AssociatedInfo> {
        let full_path = self.repo_to_full_path(repo_path);
        if info_json::is_info_json(&full_path) {
            let associated_info = info_json::get_associated_info(&full_path)?;
            Ok(associated_info.into_iter()
                .map(|(k,v)| (self.full_to_repo_path(&k).unwrap(),v))
                .collect())
        } else {
            panic!("Uh oh");
        }
    }

    pub fn listdir(&self, repo_path: &RepoPathBuf) -> std::io::Result<ListDirRecurIterator> {
        let full_path = self.repo_to_full_path(repo_path);
        // Do the first read_dir outside the Iterator to catch errors early
        let read_dir = std::fs::read_dir(full_path)?;
        Ok(ListDirRecurIterator {
            file_tree: self.clone(),
            read_dir: Some(read_dir),
            paths: VecDeque::new(),
            recurse: false,
        })
    }

    pub fn list_recursive(&self, repo_path: &RepoPathBuf) -> std::io::Result<ListDirRecurIterator> {
        let full_path = self.repo_to_full_path(repo_path);
        // Do the first read_dir outside the Iterator to catch errors early
        let read_dir = std::fs::read_dir(full_path)?;
        Ok(ListDirRecurIterator {
            file_tree: self.clone(),
            read_dir: Some(read_dir),
            paths: VecDeque::new(),
            recurse: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Read;
    use std::collections::HashSet;

    use crate::testing::testdata_path;

    fn write_file(file_root: &Path, entry_path: &str, file_contents: &str) -> std::io::Result<()> {
        let fs_path = file_root.join(entry_path);
        if let Some(parent_dir) = fs_path.parent() {
            std::fs::create_dir_all(parent_dir)?;
        }

        std::fs::write(&fs_path, file_contents)
    }

    #[test]
    fn test_open_read() -> std::io::Result<()> {
        let tempdir = tempfile::tempdir()?;
        let file_root = tempdir.path();

        write_file(file_root, "file1.mp4", "hello, world")?;
        write_file(file_root, "dir1/file2.mp4", "goodbye")?;

        let file_tree = FileTree {
            base_path: file_root.to_path_buf(),
            skip_paths: Vec::new(),
            include_non_media: true,
        };

        let mut file1 = file_tree.open_read(&RepoPathBuf::from("file1.mp4"))?;
        let mut file1_str = String::new();
        file1.read_to_string(&mut file1_str).unwrap();
        assert_eq!(file1_str, "hello, world");

        let mut file2 = file_tree.open_read(&RepoPathBuf::from("dir1/file2.mp4"))?;
        let mut file2_str = String::new();
        file2.read_to_string(&mut file2_str).unwrap();
        assert_eq!(file2_str, "goodbye");

        // Non-existent file
        let error = file_tree.open_read(&RepoPathBuf::from("dir2/file3.mp4")).err().unwrap();
        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);

        Ok(())
    }

    #[test]
    fn test_listdir() -> std::io::Result<()> {
        let tempdir = tempfile::tempdir()?;
        let file_root = tempdir.path();

        write_file(file_root, ".filer/entries.json", "{}")?;
        write_file(file_root, "file1.mp4", "hello, world")?;
        write_file(file_root, "dir1/file2.mp4", "goodbye")?;
        write_file(file_root, "dir1/no_ext", "a")?;
        
        let file_tree = FileTree {
            base_path: file_root.to_path_buf(),
            skip_paths: Vec::new(),
            include_non_media: true,
        };
        let root_contents: Vec<FsEntry> = file_tree.listdir(&RepoPathBuf::from(""))?.collect();
        assert_eq!(root_contents.len(), 2); // 1 file, 1 dir
        assert!(root_contents.iter().find(|l| l.repo_path == RepoPathBuf::from(".filer")).is_none());

        let file1_entry = root_contents.iter().find(|l| l.repo_path == RepoPathBuf::from("file1.mp4")).unwrap();
        assert_eq!(file1_entry.file_name, "file1.mp4");

        let dir1_entry = root_contents.iter().find(|l| l.repo_path == RepoPathBuf::from("dir1")).unwrap();
        assert_eq!(dir1_entry.file_name, "dir1");

        let dir1_contents: Vec<FsEntry> = file_tree.listdir(&RepoPathBuf::from("dir1"))?.collect();
        assert_eq!(dir1_contents.len(), 2);

        let file2_entry = dir1_contents.iter().find(|l| l.repo_path == RepoPathBuf::from("dir1/file2.mp4")).unwrap();
        assert_eq!(file2_entry.file_name, "file2.mp4");
        let noext_entry = dir1_contents.iter().find(|l| l.repo_path == RepoPathBuf::from("dir1/no_ext")).unwrap();
        assert_eq!(noext_entry.file_name, "no_ext");

        Ok(())
    }

    #[test]
    fn test_not_found() -> std::io::Result<()> {
        let root = testdata_path("mixed");
        let file_tree = FileTree {
            base_path: root.to_path_buf(),
            skip_paths: Vec::new(),
            include_non_media: true,
        };

        assert_eq!(
            file_tree.get_fs_entry(&RepoPathBuf::from("nonexistent"))
                .err().expect("Should be err")
                .kind(),
            std::io::ErrorKind::NotFound
        );

        assert_eq!(
            file_tree.open_read(&RepoPathBuf::from("nonexistent"))
                .err().expect("Should be err")
                .kind(),
            std::io::ErrorKind::NotFound
        );

        assert_eq!(
            file_tree.read_metadata_file(&RepoPathBuf::from("nonexistent.info.json"))
                .err().expect("Should be err")
                .kind(),
            std::io::ErrorKind::NotFound
        );

        assert_eq!(
            file_tree.listdir(&RepoPathBuf::from("nonexistent"))
                .err().expect("Should be err")
                .kind(),
            std::io::ErrorKind::NotFound
        );

        assert_eq!(
            file_tree.list_recursive(&RepoPathBuf::from("nonexistent"))
                .err().expect("Should be err")
                .kind(),
            std::io::ErrorKind::NotFound
        );

        Ok(())
    }

    #[test]
    fn test_no_escape() -> std::io::Result<()> {
        /*let tempdir = tempfile::tempdir()?;
        let file_root = tempdir.path().join("file_root");
        std::fs::create_dir(&file_root)?;

        // Create a file outside the root
        let parent_dir = file_root.parent().unwrap();
        let outside_file = parent_dir.join("outside.txt");
        std::fs::write(&outside_file, "should not be accessible")?;

        let file_tree = FileTree {
            base_path: file_root.to_path_buf(),
            skip_paths: Vec::new(),
            include_non_media: true,
        };

        // Try to access a file using ".." in the repo path
        let escape_path = RepoPathBuf::from("../outside.txt");
        // Should not find the file, or should error
        let result = file_tree.open_read(&escape_path);
        assert!(result.is_err(), "Should not be able to escape base_path");

        // Try to access using full_to_repo_path
        assert!(file_tree.full_to_repo_path(&outside_file).is_none(),
            "Should not be able to generate a repo_path from outside the root");

        let result = file_tree.get_fs_entry(&escape_path);
        assert!(result.is_err(), "Should not be able to escape base_path");

        let result = file_tree.listdir(&escape_path);
        assert!(result.is_err(), "Should not be able to escape base_path");

        let result = file_tree.list_recursive(&escape_path);
        assert!(result.is_err(), "Should not be able to escape base_path");*/

        // TODO: RepoPathBuf needs to handle ".." properly for this test to work.

        Ok(())
    }

    #[test]
    fn test_list_recursive() -> std::io::Result<()> {
        let tempdir = tempfile::tempdir()?;
        let file_root = tempdir.path();

        write_file(file_root, ".filer/entries.json", "{}")?;
        write_file(file_root, "file1.mp4", "hello, world")?;
        write_file(file_root, "dir1/file2.mp4", "goodbye")?;
        write_file(file_root, "dir1/dir2/file3.txt", "a")?;
        write_file(file_root, "dir1/dir3/file4.txt", "b")?;
        write_file(file_root, "dir4/file5.txt", "c")?;

        let file_tree = FileTree {
            base_path: file_root.to_path_buf(),
            skip_paths: Vec::new(),
            include_non_media: true,
        };
        let root_filenames: HashSet<String> = file_tree.list_recursive(&RepoPathBuf::from(""))?
            .map(|fl| fl.repo_path.0)
            .collect();
        assert_eq!(root_filenames.len(), 9); // 5 files, 4 dirs

        assert!(!root_filenames.contains(".filer"));
        assert!(root_filenames.contains("file1.mp4"));
        assert!(root_filenames.contains("dir1"));
        assert!(root_filenames.contains("dir1/file2.mp4"));
        assert!(root_filenames.contains("dir1/dir2"));
        assert!(root_filenames.contains("dir1/dir2/file3.txt"));
        assert!(root_filenames.contains("dir1/dir3"));
        assert!(root_filenames.contains("dir1/dir3/file4.txt"));
        assert!(root_filenames.contains("dir4"));
        assert!(root_filenames.contains("dir4/file5.txt"));

        // Start from a non-root dir
        let dir1_filenames: HashSet<String> = file_tree.list_recursive(&RepoPathBuf::from("dir1"))?
            .map(|fl| fl.repo_path.0)
            .collect();
        assert_eq!(dir1_filenames.len(), 5); // 3 files, 2 dirs

        assert!(dir1_filenames.contains("dir1/file2.mp4"));
        assert!(dir1_filenames.contains("dir1/dir2"));
        assert!(dir1_filenames.contains("dir1/dir2/file3.txt"));
        assert!(dir1_filenames.contains("dir1/dir3"));
        assert!(dir1_filenames.contains("dir1/dir3/file4.txt"));

        Ok(())
    }

    #[test]
    fn test_include_non_media() -> std::io::Result<()> {
        let root = testdata_path("mixed");

        let file_tree = FileTree {
            base_path: root.to_path_buf(),
            skip_paths: Vec::new(),
            include_non_media: false,
        };

        let listdir_filenames: HashSet<String> = file_tree.listdir(&RepoPathBuf::from(""))?
            .map(|fl| fl.repo_path.0)
            .collect();
        assert!(listdir_filenames.contains("plain_text.txt"));
        assert!(listdir_filenames.contains("Photos")); // Should still include dirs
        assert!(listdir_filenames.contains("Videos"));
        assert!(listdir_filenames.contains("Documents"));

        let recurse_filenames: HashSet<String> = file_tree.list_recursive(&RepoPathBuf::from(""))?
            .map(|fl| fl.repo_path.0)
            .collect();
        assert_eq!(false, recurse_filenames.contains("Other/ubuntu.torrent"));
        assert_eq!(false, recurse_filenames.contains("Other/hello.sh"));
        assert_eq!(false, recurse_filenames.contains("Other/fibonacci.csv"));
        assert_eq!(false, recurse_filenames.contains("Other/zeros.bin"));
        assert!(recurse_filenames.contains("plain_text.txt"));
        assert!(recurse_filenames.contains("Documents/lorem_ipsum.pdf"));
        assert!(recurse_filenames.contains("Videos/berlin_wall.info.json")); // Should still include metadata files
        assert!(recurse_filenames.contains("Videos/berlin_wall.mp4"));
        assert!(recurse_filenames.contains("Photos/autumn_tall.jpg"));

        Ok(())
    }

    #[test]
    fn test_skip_paths() -> std::io::Result<()> {
        let root = testdata_path("mixed");

        let file_tree = FileTree {
            base_path: root.to_path_buf(),
            skip_paths: vec!(
                root.join("Photos/autumn_tall.jpg"), // Skip one file
                root.join("Documents"), // Skip a dir
            ),
            include_non_media: true,
        };

        let recurse_filenames: HashSet<String> = file_tree.list_recursive(&RepoPathBuf::from(""))?
            .map(|fl| fl.repo_path.0)
            .collect();
        assert!(recurse_filenames.contains("page.html"));
        assert!(recurse_filenames.contains("plain_text.txt"));
        assert!(recurse_filenames.contains("Videos/berlin_wall.info.json"));
        assert!(recurse_filenames.contains("Videos/berlin_wall.mp4"));
        assert!(recurse_filenames.contains("Photos/cats_tall.jpg"));
        assert!(recurse_filenames.contains("Photos/model_tall.jpg"));
        assert!(recurse_filenames.contains("Photos/pidgeon_wide.jpg"));

        // Skipped
        assert!(!recurse_filenames.contains("Documents"));
        assert!(!recurse_filenames.contains("Documents/lorem_ipsum.md"));
        assert!(!recurse_filenames.contains("Documents/lorem_ipsum.pdf"));
        assert!(!recurse_filenames.contains("Photos/autumn_tall.jpg"));

        Ok(())
    }
}
