use crate::{catalog::Catalog, FsEntry, FileTree, Entry, RepoPathBuf};
use crate::catalog::db_entry::DbEntry;

#[derive(Clone)]
pub struct HalfEntry {
    pub fs: crate::file_tree::FsEntry,
    pub db: Option<DbEntry>,
}

pub struct ScanListing {
    pub visible: Vec<Entry>,
    pub hidden: Vec<HalfEntry>,
}

fn should_hide_entry(entry: &DbEntry) -> bool {
    let hidden_special_entry_type = entry.special_type.as_ref().is_some_and(|t| match t {
        crate::catalog::SpecialEntryType::SeriesDir => false,
        crate::catalog::SpecialEntryType::GalleryDir => false,
        crate::catalog::SpecialEntryType::MetadataFile => true,
        crate::catalog::SpecialEntryType::PreviewFile => true,
        crate::catalog::SpecialEntryType::AltFormatFile => true,
        crate::catalog::SpecialEntryType::SubtitleFile => true,
    });
    entry.deleted || entry.associated_entry.is_some() || hidden_special_entry_type
}

struct Scanner<'a> {
    catalog: &'a mut Catalog,
}

impl<'a> Scanner<'a> {
    fn list_iterator_to_result(
        &mut self,
        iterator: Box<dyn Iterator<Item = FsEntry>>,
    ) -> Result<ScanListing, Box<dyn std::error::Error>> {

        let mut visible = Vec::new();
        let mut hidden = Vec::new();

        for child in iterator {
            let id_maybe = self.catalog.path_to_id(&child.repo_path);
            let db_entry_maybe = id_maybe.and_then(|id| self.catalog.get_by_id(id));

            // Check whether this file should be hidden
            let should_be_hidden = child.is_metadata_file ||
                db_entry_maybe.as_ref().map(|e| should_hide_entry(e)).unwrap_or(false);

            if should_be_hidden {
                hidden.push(HalfEntry {
                    fs: child,
                    db: db_entry_maybe,
                });
            } else {
                let db_entry = self.catalog.get_or_create(&child);
                visible.push(Entry {
                    fs: child,
                    db: db_entry,
                });
            }
        }

        Ok(ScanListing { visible, hidden })
    }
}

pub fn listdir(catalog: &mut Catalog, file_tree: &FileTree, path: &RepoPathBuf) -> Result<ScanListing, Box<dyn std::error::Error>> {
    let mut scanner = Scanner { catalog };
    scanner.list_iterator_to_result(Box::new(file_tree.listdir(path)?))
}

pub fn list_recursive(catalog: &mut Catalog, file_tree: &FileTree, root: &RepoPathBuf) -> Result<ScanListing, Box<dyn std::error::Error>> {
    let mut scanner = Scanner { catalog };
    scanner.list_iterator_to_result(Box::new(file_tree.list_recursive(root)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testing;

    fn get_visible(visible: &Vec<Entry>, path: &RepoPathBuf) -> Option<Entry> {
        for entry in visible {
            if &entry.fs.repo_path == path {
                return Some(entry.clone());
            }
        }
        None
    }

    fn get_hidden(hidden: &Vec<HalfEntry>, path: &RepoPathBuf) -> Option<HalfEntry> {
        for lmbe in hidden {
            if &lmbe.fs.repo_path == path {
                return Some(lmbe.clone());
            }
        }
        None
    }

    #[test]
    fn test_listdir() -> Result<(), Box<dyn std::error::Error>> {
        let root = testing::testdata_path("mixed");
        let (_tempdir, stash) = testing::tempdir_vault(&root)?;

        let mut catalog = stash.open_catalog()?;
        let listing = listdir(&mut catalog, &stash.new_file_tree(), &RepoPathBuf::from("Videos"))?;

        // Video file should be visible
        let mp4_entry = get_visible(&listing.visible, &RepoPathBuf::from("Videos/berlin_wall.mp4"))
            .expect("get_visible");
        assert!(mp4_entry.db.id >= 0);

        // Associated metadata file should be hidden
        let info_json_hentry = get_hidden(&listing.hidden, &RepoPathBuf::from("Videos/berlin_wall.info.json"))
            .expect("get_hidden");
        assert!(info_json_hentry.db.is_none());

        // Make sure Entry was created for mp4, was NOT created for info.json file
        assert!(catalog.contains_path(&RepoPathBuf::from("Videos/berlin_wall.mp4")));
        assert_eq!(false, catalog.contains_path(&RepoPathBuf::from("Videos/berlin_wall.info.json")));

        Ok(())
    }
}
