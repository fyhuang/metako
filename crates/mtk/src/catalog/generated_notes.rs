use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;

use crate::Entry;

use super::{Catalog, DbEntry};

/// Helpers for checking and updating generated notes
///
/// Generated notes are stored in groups in the Entry's notes_generated. The group structure
/// looks like this:
///
/// Key | Value
/// ----|------
/// group_name::__last_update | timestamp
/// group_name::key1 | value1
/// group_name::key2 | value2

fn get_last_update_key(group_name: &str) -> String {
    if group_name.is_empty() {
        panic!("group_name cannot be empty");
    }
    format!("{}::__last_update", group_name)
}

/// Check if the generated group needs to be updated (if the file's modtime is newer than the group's last update time).
pub fn needs_update(entry: &Entry, group_name: &str) -> bool {
    let last_update_key = get_last_update_key(group_name);
    if let Some(last_update_value) = entry.db.notes_generated.get(&last_update_key) {
        let last_update_secs = last_update_value.as_u64().expect("__last_update as_u64");
        let mod_time_secs = entry.fs.mod_time.timestamp() as u64;
        return mod_time_secs > last_update_secs;
    } else {
        true
    }
}

/// Update the generated group with the current time and the new values. Each sub-key in the container
/// will be stored as a separate key in the group.
pub fn update<T: Serialize>(catalog: &mut Catalog, entry_id: i64, group_name: &str, container: &T) {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("duration_since")
        .as_secs();
    let last_update_key = get_last_update_key(group_name);

    let container_map = {
        let container_value = serde_json::to_value(container).expect("to_value container");
        match container_value {
            serde_json::Value::Object(map) => map,
            _ => panic!("container should be an object"),
        }
    };

    catalog.update_notes_with(
        entry_id,
        super::sqlite_catalog::WhichNotes::Generated,
        |notes| {
            notes[&last_update_key] = json!(current_time);
            for (key, value) in container_map {
                let full_key = format!("{}::{}", group_name, key);

                // TODO: implement full RFC-7396 merge patch
                if value.is_null() {
                    let notes_map = notes.as_object_mut().expect("notes as_object_mut");
                    notes_map.remove(&full_key);
                } else {
                    notes[&full_key] = value;
                }
            }
        },
    );
}

/// Read the generated group from the entry's notes_generated field.
pub fn read<T: DeserializeOwned>(entry: &DbEntry, group_name: &str) -> Option<T> {
    let last_update_key = get_last_update_key(group_name);

    if let Some(generated_map) = entry.notes_generated.as_object() {
        let mut out_map = serde_json::Map::new();
        for (key, value) in generated_map {
            if key == &last_update_key {
                continue;
            }

            if key.starts_with(&format!("{}::", group_name)) {
                let sub_key = key.split("::").nth(1).unwrap_or("");
                out_map.insert(sub_key.to_string(), value.clone());
            }
        }

        if out_map.is_empty() {
            None
        } else {
            Some(serde_json::from_value(serde_json::Value::Object(out_map)).expect("from_value"))
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{testing, RepoPathBuf};

    use super::*;

    #[test]
    fn test_get_last_update_key() {
        assert_eq!(get_last_update_key("group1"), "group1::__last_update");
        assert_eq!(get_last_update_key("another_group"), "another_group::__last_update");
    }

    #[test]
    #[should_panic]
    fn test_get_last_update_key_empty_group_name() {
        get_last_update_key("");
    }

    #[test]
    fn test_needs_update_older() {
        let mut fs_entry = testing::test_fs_entry("test.mp4");
        fs_entry.mod_time = chrono::DateTime::<chrono::Utc>::from_timestamp(150, 0).unwrap();

        let mut db_entry = DbEntry::default(123, RepoPathBuf::from("test.mp4"));
        db_entry.notes_generated = json!({
            "group1::__last_update": 100
        });

        let entry = Entry {
            fs: fs_entry,
            db: db_entry,
        };

        assert!(needs_update(&entry, "group1"));
    }

    #[test]
    fn test_needs_update_newer() {
        let mut fs_entry = testing::test_fs_entry("test.mp4");
        fs_entry.mod_time = chrono::DateTime::<chrono::Utc>::from_timestamp(50, 0).unwrap();

        let mut db_entry = DbEntry::default(123, RepoPathBuf::from("test.mp4"));
        db_entry.notes_generated = json!({
            "group1::__last_update": 100
        });

        let entry = Entry {
            fs: fs_entry,
            db: db_entry,
        };

        assert_eq!(false, needs_update(&entry, "group1"));
    }

    #[test]
    fn test_needs_update_first_time() {
        let mut fs_entry = testing::test_fs_entry("test.mp4");
        fs_entry.mod_time = chrono::DateTime::<chrono::Utc>::from_timestamp(150, 0).unwrap();

        let mut db_entry = DbEntry::default(123, RepoPathBuf::from("test.mp4"));
        db_entry.notes_generated = json!({});

        let entry = Entry {
            fs: fs_entry,
            db: db_entry,
        };

        assert!(needs_update(&entry, "group1"));
    }

    #[test]
    fn test_read_existing_group() {
        let mut db_entry = DbEntry::default(123, RepoPathBuf::from("test.mp4"));
        db_entry.notes_generated = json!({
            "group1::__last_update": 100,
            "group1::key1": "value1",
            "group1::key2": "value2"
        });

        let result: Option<serde_json::Value> = read(&db_entry, "group1");
        let expected = json!({
            "key1": "value1",
            "key2": "value2"
        });

        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_read_non_existing_group() {
        let mut db_entry = DbEntry::default(123, RepoPathBuf::from("test.mp4"));
        db_entry.notes_generated = json!({
            "group1::__last_update": 100,
            "group1::key1": "value1",
            "group1::key2": "value2"
        });

        let result: Option<serde_json::Value> = read(&db_entry, "group2");
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_empty_nets_generated() {
        let mut db_entry = DbEntry::default(123, RepoPathBuf::from("test.mp4"));
        db_entry.notes_generated = json!({});

        let result: Option<serde_json::Value> = read(&db_entry, "group1");
        assert_eq!(result, None);
    }

    #[test]
    fn test_update() {
        let conn = testing::in_memory_conn("");
        let mut catalog = Catalog::from_conn(conn);

        let fs_entry = testing::test_fs_entry("test.mp4");
        let entry_id = catalog.get_or_create(&fs_entry).id;

        let container = json!({
            "string_key": "value1",
            "u64_key": 42,
            "float_key": 3.14,
            "null_key": null,
        });
        update(&mut catalog, entry_id, "group1", &container);

        // Read out the Entry
        let entry = catalog.get_by_id(entry_id).expect("get entry");
        let generated_map = entry.notes_generated.as_object().expect("as_object");
        assert!(generated_map.contains_key("group1::__last_update"));  // the actual value is not deterministic
        assert_eq!(&generated_map["group1::string_key"], &json!("value1"));
        assert_eq!(&generated_map["group1::u64_key"], &json!(42));
        assert_eq!(&generated_map["group1::float_key"], &json!(3.14));
        assert_eq!(false, generated_map.contains_key("group1::null_key"));
    }
}
