use std::str::FromStr;

use crate::CatalogError;

pub fn enum_from_sql<'a, T: FromStr>(
    sql_value: rusqlite::types::ValueRef<'a>,
) -> Result<Option<T>, rusqlite::Error> {
    match sql_value {
        rusqlite::types::ValueRef::Null => Ok(None),
        rusqlite::types::ValueRef::Text(byte_slice) => {
            let s = std::str::from_utf8(byte_slice)?;
            Ok(Some(
                T::from_str(s).map_err(|_| rusqlite::Error::InvalidQuery)?,
            ))
        }
        _ => Err(rusqlite::Error::InvalidColumnType(
            0,
            "special_type_from_sql expects sqlite type TEXT".to_string(),
            sql_value.data_type(),
        )),
    }
}

pub fn deser_value_from_row<T>(row: &rusqlite::Row, idx: usize) -> Result<T, CatalogError>
where
    T: serde::de::DeserializeOwned,
{
    let value_ref = row.get_ref(idx)?;
    match value_ref {
        rusqlite::types::ValueRef::Null => {
            // This is a NOT NULL column, so this shouldn't happen
            Ok(serde_json::from_str("{}")?)
        }
        rusqlite::types::ValueRef::Text(byte_slice) => {
            let s = std::str::from_utf8(byte_slice)?;
            Ok(serde_json::from_str(s)?)
        }
        _ => Err(CatalogError::from(rusqlite::Error::InvalidColumnType(
            idx,
            "deser_value_from_row expects TEXT".to_string(),
            value_ref.data_type(),
        ))),
    }
}

pub fn check_or_set_app_id(conn: &rusqlite::Connection, expected_app_id: i32) -> Result<(), CatalogError> {
    let app_id: i32 = conn
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .unwrap();
    if app_id == 0 {
        // Set the app ID
        conn.pragma_update(None, "application_id", &expected_app_id)?;
    } else if app_id != expected_app_id {
        return Err(CatalogError::db_check_error(&format!(
            "DB has unknown application_id {}", app_id
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{catalog::SpecialEntryType, testing};

    use super::*;

    #[test]
    fn test_enum_from_sql() {
        let text_bytes = "SeriesDir".as_bytes();
        let sql_value = rusqlite::types::ValueRef::Text(text_bytes);
        assert_eq!(
            enum_from_sql::<SpecialEntryType>(sql_value)
                .unwrap()
                .expect("should have been Some"),
            SpecialEntryType::SeriesDir
        );
    }

    #[test]
    fn test_deser_value_from_row() {
        let conn = testing::in_memory_conn("");
        conn.execute_batch(
            "
            CREATE TABLE entries (
                notes_user TEXT NOT NULL DEFAULT '{}'
            );

            INSERT INTO entries (
                notes_user
            ) VALUES (
                '{ \"title\": \"test file\" }'
            );
            ",
        )
        .unwrap();

        let notes: serde_json::Value = conn
            .query_row_and_then("SELECT notes_user FROM entries", [], |row| {
                deser_value_from_row::<serde_json::Value>(row, 0)
            })
            .expect("should have been Ok");

        assert_eq!(notes["title"], "test file");
    }
}
