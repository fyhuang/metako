use crate::catalog::Catalog;
use crate::{Entry, RepoPathBuf};
use crate::FileTree;

pub fn search(
    file_tree: &FileTree,
    catalog: &mut Catalog,
    search_root: &RepoPathBuf,
    query: &str,
) -> Vec<Entry> {
    let re_string = format!("(?i){}", query);
    let re = regex::Regex::new(&re_string).unwrap();
    println!("Searching for \"{}\"", query);

    let mut results = vec![];

    for fs_entry in file_tree.list_recursive(search_root).unwrap() {
        // Search skips metadata files
        if fs_entry.is_metadata_file {
            continue;
        }

        let db_entry = catalog.get_or_create(&fs_entry);
        let is_match = if re.is_match(&fs_entry.file_name) {
            true
        } else {
            let merged_text = db_entry.notes.to_string();
            re.is_match(&merged_text)
        };

        if is_match {
            results.push(Entry { fs: fs_entry, db: db_entry });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{catalog::sqlite_catalog::WhichNotes, testing};
    use serde_json::json;

    fn results_contain(entries: &[Entry], repo_path_str: &str) -> bool {
        for entry in entries {
            if entry.fs.repo_path.0 == repo_path_str {
                return true;
            }
        }
        false
    }

    #[test]
    fn test_search() {
        let conn = testing::in_memory_conn("");
        let mut catalog = Catalog::from_conn(conn);

        let root = testing::testdata_path("mixed");
        let file_tree = FileTree::new(&root, Vec::new(), true);

        let r_autumn = catalog.get_or_create(
            &file_tree
                .get_fs_entry(&RepoPathBuf::from("Photos/autumn_tall.jpg"))
                .expect("get_fs_entry"),
        );
        catalog.set_single_note(r_autumn.id, WhichNotes::User, "title", json!("Fall colors"));

        let r_berlin = catalog.get_or_create(
            &file_tree
                .get_fs_entry(&RepoPathBuf::from("Videos/berlin_wall.mp4"))
                .expect("get_fs_entry"),
        );
        catalog.set_single_note(r_berlin.id, WhichNotes::Generated, "length", json!("1:23"));

        // Search by filename
        let s_plain_text = search(
            &file_tree,
            &mut catalog,
            &RepoPathBuf::from(""),
            "plain_text",
        );
        assert!(results_contain(&s_plain_text, "plain_text.txt"));

        // Search by user notes
        let s_fall = search(&file_tree, &mut catalog, &RepoPathBuf::from(""), "Fall");
        assert!(results_contain(&s_fall, "Photos/autumn_tall.jpg"));

        // Search by non-user notes (from generated)
        let s_berlin = search(&file_tree, &mut catalog, &RepoPathBuf::from(""), "23");
        assert!(results_contain(&s_berlin, "Videos/berlin_wall.mp4"));
    }

    #[test]
    fn test_search_no_metadata() {
        let conn = testing::in_memory_conn("");
        let mut catalog = Catalog::from_conn(conn);

        let root = testing::testdata_path("mixed");
        let file_tree = FileTree::new(&root, Vec::new(), true);

        // Should not contain metadata files
        let s_none = search(&file_tree, &mut catalog, &RepoPathBuf::from(""), "berlin");
        assert!(results_contain(&s_none, "Videos/berlin_wall.mp4"));
        assert!(!results_contain(&s_none, "Videos/berlin_wall.info.json"));
    }
}
