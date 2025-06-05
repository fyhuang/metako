pub mod filetype;
mod sqlite;

// Helpers for testing
pub mod testing;

pub mod error;
pub use error::CatalogError;

// File tree
pub mod repo_path;
pub use repo_path::RepoPathBuf;

pub mod file_tree;
pub use file_tree::{FsEntry, FileTree};

// Catalog
pub mod catalog;

#[derive(Clone)]
pub struct Entry {
    pub fs: file_tree::FsEntry,
    pub db: catalog::DbEntry,
}
