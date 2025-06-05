use std::path::Path;

use rusqlite::OptionalExtension;

use super::db_entry::DbEntry;
use super::SpecialEntryType;

use crate::error::{CatalogError, InnerError};
use crate::sqlite;
use crate::{FsEntry, RepoPathBuf};

const APP_ID: i32 = 0x53747368; // Stsh

const ALL_COLUMN_NAMES: &[&'static str] = &[
    "entry_id",
    "repo_path",
    "deleted",
    "special_type",
    "associated_entry",
    "notes_user",
    "notes_external",
    "notes_generated",
    "notes",
];

fn row_to_entry(row: &rusqlite::Row) -> Result<DbEntry, CatalogError> {
    Ok(DbEntry {
        id: row.get(0)?,
        repo_path: RepoPathBuf::from(row.get_ref(1)?.as_str()?),
        deleted: row.get(2)?,
        special_type: sqlite::enum_from_sql::<SpecialEntryType>(row.get_ref(3)?)?,
        associated_entry: row.get(4)?,
        notes_user: sqlite::deser_value_from_row(row, 5)?,
        notes_external: sqlite::deser_value_from_row(row, 6)?,
        notes_generated: sqlite::deser_value_from_row(row, 7)?,
        notes: sqlite::deser_value_from_row(row, 8)?,
    })
}

fn get_by_path(tx: &rusqlite::Transaction, repo_path: &RepoPathBuf) -> Option<DbEntry> {
    let result = tx.query_row_and_then(
        &format!(
            "SELECT {} FROM entries WHERE repo_path = ?",
            ALL_COLUMN_NAMES.join(",")
        ),
        &[repo_path.as_str()],
        row_to_entry,
    );
    match result {
        Ok(entry) => Some(entry),
        Err(e) => match e.error {
            InnerError::RusqliteError(rusqlite::Error::QueryReturnedNoRows) => None,
            _ => panic!("Error Catalog::get_by_path: {}", e),
        },
    }
}

pub enum WhichNotes {
    User,
    External,
    Generated,
}

pub struct Catalog {
    conn: rusqlite::Connection,
}

impl Catalog {
    pub fn open(db_filename: &Path) -> Result<Self, CatalogError> {
        let conn = rusqlite::Connection::open(db_filename)?;
        Ok(Self::from_conn(conn))
    }

    pub fn from_conn(conn: rusqlite::Connection) -> Self {
        Self::init_conn(&conn);
        Self { conn: conn }
    }

    fn init_conn(conn: &rusqlite::Connection) {
        // Make sure schema is up-to-date
        sqlite::check_or_set_app_id(conn, APP_ID).unwrap();

        let user_version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        if user_version == 0 {
            // Create tables
            conn.execute_batch(
                "
                CREATE TABLE entries (
                    entry_id INTEGER PRIMARY KEY,
                    repo_path TEXT NOT NULL,
                    deleted BOOL NOT NULL DEFAULT FALSE,

                    special_type TEXT,
                    associated_entry INTEGER,

                    notes_user TEXT NOT NULL DEFAULT '{}',
                    notes_external TEXT NOT NULL DEFAULT '{}',
                    notes_generated TEXT NOT NULL DEFAULT '{}',

                    notes TEXT GENERATED ALWAYS AS (
                        json_patch(
                            json_patch(
                                json_patch('{}', notes_generated),
                                notes_external
                            ),
                            notes_user
                        )
                    ) VIRTUAL,

                    UNIQUE(repo_path)
                );
            ",
            )
            .unwrap();
            conn.pragma_update(None, "user_version", &1).unwrap();
        } else if user_version == 1 {
            // Latest schema version
        } else {
            panic!("Unknown user_version {}", user_version);
        }
    }

    pub fn get_by_id(&self, entry_id: i64) -> Option<DbEntry> {
        let result = self.conn.query_row_and_then(
            "SELECT * FROM entries WHERE entry_id = ?",
            &[&entry_id],
            row_to_entry,
        );
        match result {
            Ok(entry) => Some(entry),
            Err(e) => match e.error {
                InnerError::RusqliteError(rusqlite::Error::QueryReturnedNoRows) => None,
                _ => panic!("Error Catalog::get_by_id: {}", e),
            },
        }
    }

    pub fn path_to_id(&self, repo_path: &RepoPathBuf) -> Option<i64> {
        self.conn
            .query_row(
                "SELECT entry_id FROM entries WHERE repo_path = ? LIMIT 1",
                &[repo_path.as_str()],
                |row| row.get::<usize, i64>(0),
            )
            .optional()
            .expect("path_to_id")
    }

    pub fn contains_path(&self, repo_path: &RepoPathBuf) -> bool {
        self.path_to_id(repo_path).is_some()
    }

    pub fn get_or_create(&mut self, fs_entry: &FsEntry) -> DbEntry {
        let tx = self.conn.transaction().unwrap();

        let existing_row = get_by_path(&tx, &fs_entry.repo_path);
        if let Some(row) = existing_row {
            // Check if the row is deleted or not
            if row.deleted {
                // Update the row to undelete it
                tx.execute(
                    "UPDATE entries SET deleted = FALSE WHERE entry_id = ?1",
                    (row.id,),
                )
                .unwrap();
            }
            tx.commit().unwrap();

            return get_by_path(&self.conn.transaction().unwrap(), &fs_entry.repo_path).unwrap();
        } else {
            // Create and return the new entry
            tx.execute(
                "INSERT OR IGNORE INTO entries (repo_path) VALUES (?1)",
                (
                    fs_entry.repo_path.as_str(),
                ),
            )
            .unwrap();
            tx.commit().unwrap();

            return get_by_path(&self.conn.transaction().unwrap(), &fs_entry.repo_path).unwrap();
        }
    }

    pub fn update_notes_with(
        &mut self,
        id: i64,
        which: WhichNotes,
        updater: impl FnOnce(&mut serde_json::Value),
    ) {
        let tx = self.conn.transaction().unwrap();
        let col_name = match which {
            WhichNotes::User => "notes_user",
            WhichNotes::External => "notes_external",
            WhichNotes::Generated => "notes_generated",
        };

        let mut notes = tx
            .query_row_and_then(
                &format!("SELECT {} FROM entries WHERE entry_id = ?1", col_name),
                (id,),
                |row| sqlite::deser_value_from_row::<serde_json::Value>(&row, 0),
            )
            .expect("query should succeed");

        updater(&mut notes);

        let new_notes_str = serde_json::to_string(&notes).unwrap();
        tx.execute(
            &format!(
                "UPDATE entries SET {} = json(?1) WHERE entry_id = ?2",
                col_name
            ),
            (new_notes_str, id),
        )
        .expect("update should succeed");

        tx.commit().unwrap();
    }

    pub fn set_single_note(
        &mut self,
        id: i64,
        which: WhichNotes,
        key: &str,
        value: serde_json::Value,
    ) {
        self.update_notes_with(id, which, |note| {
            note[key] = value;
        })
    }

    pub fn set_notes_json(&mut self, id: i64, which: WhichNotes, json_str: &str) {
        let col_name = match which {
            WhichNotes::User => "notes_user",
            WhichNotes::External => "notes_external",
            WhichNotes::Generated => "notes_generated",
        };
        self.conn
            .execute(
                &format!(
                    "UPDATE entries SET {} = json(?1) WHERE entry_id = ?2",
                    col_name
                ),
                (json_str, id),
            )
            .expect("update should succeed");
    }

    // Testing/debugging functions
    #[allow(dead_code)]
    pub(crate) fn print_all_entries(&mut self) {
        let tx = self.conn.transaction().expect("transaction");
        let mut stmt = tx.prepare("SELECT entry_id, repo_path, special_type, associated_entry FROM entries").expect("prepare");
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0).expect("get");
            let repo_path: String = row.get(1).expect("get");
            let special_type: Option<String> = row.get(2).expect("get");
            let associated_entry: Option<i64> = row.get(3).expect("get");
            Ok((id, repo_path, special_type, associated_entry))
        }).expect("query_map");
        for row in rows {
            let (id, repo_path, special_type, associated_entry) = row.expect("row");
            println!("entry: id={} repo_path={} special_type={:?} associated_entry={:?}", id, repo_path, special_type, associated_entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{self, in_memory_conn};

    fn make_fixtures(conn: &rusqlite::Connection) {
        Catalog::init_conn(conn);
        conn.execute_batch(
            "INSERT INTO entries (
                entry_id,
                repo_path
            ) VALUES (
                1,
                \"dir1/file1\"
            );
            
            INSERT INTO entries
                (entry_id, repo_path, notes_user)
            VALUES
                (2, \"dir1/file2\", json('{ \"title\": \"file2 title\" }'))
            ",
        )
        .unwrap();
    }

    #[test]
    fn test_init() {
        let conn = in_memory_conn("test_init");
        let test_conn = in_memory_conn("test_init");
        Catalog::from_conn(conn);

        test_conn
            .query_row(
                "SELECT COUNT(*) FROM entries WHERE notes_user IS NOT NULL",
                [],
                |row| row.get::<usize, i32>(0),
            )
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_init_wrong_appid() {
        let conn = in_memory_conn("test_init_wrong_appid");
        conn.pragma_update(None, "application_id", &0x12345678)
            .unwrap();
        Catalog::from_conn(conn);
    }

    #[test]
    #[should_panic]
    fn test_init_wrong_schema_version() {
        let conn = in_memory_conn("test_init_wrong_schema_version");
        conn.pragma_update(None, "user_version", &999).unwrap();
        Catalog::from_conn(conn);
    }

    #[test]
    fn test_get_by_id() {
        let conn = in_memory_conn("test_get_by_id");
        make_fixtures(&conn);

        let catalog = Catalog::from_conn(conn);
        catalog.get_by_id(1).expect("is not None");
    }

    #[test]
    fn test_path_to_id() {
        let conn = in_memory_conn("");
        make_fixtures(&conn);

        let catalog = Catalog::from_conn(conn);
        assert_eq!(catalog.path_to_id(&RepoPathBuf::from("dir1/file1")), Some(1));
        assert_eq!(catalog.path_to_id(&RepoPathBuf::from("dir1/file2")), Some(2));
        assert_eq!(catalog.path_to_id(&RepoPathBuf::from("doesnt/exist")), None);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_get_or_create__exists() {
        let conn = testing::in_memory_conn("");
        make_fixtures(&conn);

        let mut catalog = Catalog::from_conn(conn);
        let fs_entry = testing::test_fs_entry("dir1/file2");
        let row = catalog.get_or_create(&fs_entry);

        assert_eq!(row.repo_path, fs_entry.repo_path);
        assert_eq!(row.id, 2); // Make sure it didn't create a new row
        assert_eq!(row.notes_user["title"], "file2 title");
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_get_or_create__new() {
        let conn = testing::in_memory_conn("");

        let mut catalog = Catalog::from_conn(conn);
        let fs_entry = testing::test_fs_entry("dir1/file2");
        let row = catalog.get_or_create(&fs_entry);

        assert_eq!(row.repo_path, fs_entry.repo_path);
        assert_eq!(row.id, 1);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_get_or_create__deleted() {
        let conn = testing::in_memory_conn("");
        Catalog::init_conn(&conn);
        conn.execute_batch(
            "INSERT INTO entries (
                entry_id,
                repo_path,
                deleted
            ) VALUES (
                1,
                \"dir1/file1\",
                TRUE
            )",
        )
        .unwrap();

        let mut catalog = Catalog::from_conn(conn);
        let fs_entry = testing::test_fs_entry("dir1/file1");
        let row = catalog.get_or_create(&fs_entry);

        assert_eq!(row.repo_path, fs_entry.repo_path);
        assert_eq!(row.id, 1); // Make sure it didn't create a new row
        assert_eq!(row.deleted, false);
    }

    #[test]
    fn test_set_single_note() {
        let conn = testing::in_memory_conn("set_single_note");
        make_fixtures(&conn);

        let mut catalog = Catalog::from_conn(conn);
        catalog.set_single_note(
            1,
            WhichNotes::User,
            "title",
            serde_json::Value::String("file1".to_string()),
        );
        assert_eq!(catalog.get_by_id(1).unwrap().notes_user["title"], "file1");

        catalog.set_single_note(
            1,
            WhichNotes::External,
            "description",
            serde_json::Value::String("test".to_string()),
        );
        assert_eq!(
            catalog.get_by_id(1).unwrap().notes_external["description"],
            "test"
        );

        catalog.set_single_note(
            1,
            WhichNotes::Generated,
            "rating",
            serde_json::Value::Number(serde_json::Number::from(100)),
        );
        assert_eq!(catalog.get_by_id(1).unwrap().notes_generated["rating"], 100);

        // Make sure merging works
        let row = catalog.get_by_id(1).unwrap();
        assert_eq!(row.description(), "test");
    }

    #[test]
    fn test_set_notes_json() {
        let conn = testing::in_memory_conn("set_notes_json");
        make_fixtures(&conn);

        let mut catalog = Catalog::from_conn(conn);

        catalog.set_notes_json(1, WhichNotes::User, "{ \"title\": \"file1\" }");
        assert_eq!(catalog.get_by_id(1).unwrap().notes_user["title"], "file1");

        catalog.set_notes_json(1, WhichNotes::External, "{ \"description\": \"test\" }");
        assert_eq!(
            catalog.get_by_id(1).unwrap().notes_external["description"],
            "test"
        );

        catalog.set_notes_json(1, WhichNotes::Generated, "{ \"rating\": 100 }");
        assert_eq!(catalog.get_by_id(1).unwrap().notes_generated["rating"], 100);

        // Make sure merging works
        let row = catalog.get_by_id(1).unwrap();
        assert_eq!(row.description(), "test");
    }
}
