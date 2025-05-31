use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize)]
pub struct RepoPathBuf(pub String);

impl From<&str> for RepoPathBuf {
    fn from(input: &str) -> Self {
        if input.len() > 0 && input.as_bytes()[0] == b'/' {
            Self(input[1..].to_string())
        } else {
            Self(input.to_string())
        }
    }
}

impl From<&String> for RepoPathBuf {
    fn from(input: &String) -> Self { Self::from(input.as_str()) }
}

impl From<&Path> for RepoPathBuf {
    fn from(input: &Path) -> Self {
        Self::from(input.to_str().unwrap())
    }
}

impl std::fmt::Display for RepoPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl RepoPathBuf {
    pub fn as_str(&self) -> &str { &self.0 }
    
    // TODO(fyhuang): tests
    pub fn is_descendant_of(&self, parent_path: &RepoPathBuf) -> bool {
        // TODO(fyhuang): implement something a bit more sophisticated?
        self.0.starts_with(&parent_path.0)
    }

    pub fn parent(&self) -> Option<RepoPathBuf> {
        Path::new(&self.0).parent().and_then(|p| p.to_str()).map(|p| RepoPathBuf::from(p))
    }

    pub fn parent_or_empty(&self) -> RepoPathBuf {
        self.parent().unwrap_or(RepoPathBuf::from(""))
    }

    pub fn file_name<'a>(&'a self) -> &'a str {
        Path::new(&self.0).file_name().and_then(|oss| oss.to_str())
            .unwrap_or("")
    }

    pub fn to_full_path(&self, base_path: &Path) -> PathBuf {
        base_path.join(Path::new(&self.0))
    }

    pub fn from_full_path(base_path: &Path, file_path: &Path) -> Option<RepoPathBuf> {
        if (base_path.is_absolute() && !file_path.is_absolute()) ||
            (!base_path.is_absolute() && file_path.is_absolute()) {
                panic!("base_path and file_path must both be absolute or relative");
        }

        file_path.strip_prefix(base_path).ok()
            .and_then(|p| p.to_str())
            .map(|p| RepoPathBuf(p.to_string()))
    }
    
    pub fn join(&self, file_name: &str) -> RepoPathBuf {
        assert!(!file_name.is_empty());

        let mut result_str = self.0.to_string();
        if result_str.len() > 0 {
            result_str.push('/');
        }
        result_str.push_str(file_name);
        RepoPathBuf(result_str)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buf_from() {
        // From<&String>
        assert_eq!(
            RepoPathBuf::from("a.txt"),
            RepoPathBuf::from(&String::from("a.txt"))
        );

        // From<&Path>
        assert_eq!(
            RepoPathBuf::from("dir1/file1"),
            RepoPathBuf::from(Path::new("dir1/file1"))
        );
    }

    #[test]
    fn test_is_descendant_of() {
        assert!(RepoPathBuf::from("a/b/c.txt")
            .is_descendant_of(&RepoPathBuf::from("a/b")));
        assert!(RepoPathBuf::from("a/b/c.txt")
            .is_descendant_of(&RepoPathBuf::from("a/b/")));
        assert!(RepoPathBuf::from("a/b/c.txt")
            .is_descendant_of(&RepoPathBuf::from("a")));
        assert!(RepoPathBuf::from("a/b/c.txt")
            .is_descendant_of(&RepoPathBuf::from("a/")));
        assert!(RepoPathBuf::from("a/b/c.txt")
            .is_descendant_of(&RepoPathBuf::from("")));

        assert_eq!(false, RepoPathBuf::from("a/b/c.txt")
            .is_descendant_of(&RepoPathBuf::from("a/b/d")));

        assert!(RepoPathBuf::from("a.txt")
            .is_descendant_of(&RepoPathBuf::from("")));

        // TODO(fyhuang): fix this
        //assert_eq!(false, RepoPathBuf::from("a/b/c.txt").is_descendant_of("a/b/c"));
    }

    #[test]
    fn test_parent() {
        assert_eq!(
            RepoPathBuf::from("a/b/c.txt").parent().unwrap(),
            RepoPathBuf::from("a/b")
        );

        assert_eq!(
            RepoPathBuf::from("a/b.txt").parent().unwrap(),
            RepoPathBuf::from("a")
        );

        assert_eq!(
            RepoPathBuf::from("a.txt").parent().unwrap(),
            RepoPathBuf::from("")
        );

        assert!(RepoPathBuf::from("").parent().is_none());
        assert_eq!(
            RepoPathBuf::from("").parent_or_empty(),
            RepoPathBuf::from("")
        );
    }

    #[test]
    fn test_file_name() {
        assert_eq!(RepoPathBuf::from("a/b/c.txt").file_name(), "c.txt");
        assert_eq!(RepoPathBuf::from("a/b").file_name(), "b");
        assert_eq!(RepoPathBuf::from("a/b/").file_name(), "b");
        assert_eq!(RepoPathBuf::from("a").file_name(), "a");
        assert_eq!(RepoPathBuf::from("").file_name(), "");
    }

    #[test]
    fn test_to_full_path() {
        assert_eq!(
            RepoPathBuf::from("simple.txt").to_full_path(Path::new("/")),
            Path::new("/simple.txt")
        );

        assert_eq!(
            RepoPathBuf::from("my/file.txt").to_full_path(Path::new("/file/root")),
            Path::new("/file/root/my/file.txt")
        );

        assert_eq!(
            RepoPathBuf::from("my/file.txt").to_full_path(Path::new("/trailing/slash/")),
            Path::new("/trailing/slash/my/file.txt")
        );

        assert_eq!(
            RepoPathBuf::from("/file2.txt").to_full_path(Path::new("/file/root")),
            Path::new("/file/root/file2.txt")
        );
    }

    #[test]
    fn test_full_to_repo() {
        assert_eq!(
            RepoPathBuf::from_full_path(Path::new("/"), Path::new("/simple.txt")).unwrap(),
            RepoPathBuf::from("simple.txt")
        );

        assert_eq!(
            RepoPathBuf::from_full_path(Path::new("/f/r/"), Path::new("/f/r/file.txt")).unwrap(),
            RepoPathBuf::from("file.txt")
        );

        assert_eq!(
            RepoPathBuf::from_full_path(Path::new("/f/r/"), Path::new("/f/r/dir1/nested.txt")).unwrap(),
            RepoPathBuf::from("dir1/nested.txt")
        );
    }

    #[test]
    fn test_join() {
        assert_eq!(
            RepoPathBuf::from("").join("a.txt"),
            RepoPathBuf::from("a.txt")
        );

        assert_eq!(
            RepoPathBuf::from("a").join("b.txt"),
            RepoPathBuf::from("a/b.txt")
        );

        assert_eq!(
            RepoPathBuf::from("a/b").join("c.txt"),
            RepoPathBuf::from("a/b/c.txt")
        );

        assert_eq!(
            RepoPathBuf::from("a/b").join("d"),
            RepoPathBuf::from("a/b/d")
        );
    }
}
