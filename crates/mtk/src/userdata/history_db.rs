use std::path::Path;

use serde::Serialize;

use rusqlite::{params, Connection, Row};
use rusqlite::Result;
use rusqlite::OptionalExtension;

#[derive(Clone, Debug, Serialize)]
pub struct VideoHistory {
    pub farthest_ts: i64, // in seconds
    pub farthest_ts_ratio: f32, // % of video length (0-100)
    pub farthest_ts_date: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ViewHistory {
    pub entry_id: i64,
    pub last_viewed_date: Option<String>, // TODO(fyhuang): better-typed datetimes

    // Video watch history
    pub video_history: Option<VideoHistory>,

    // TODO: documents. Last read page, etc.
}

impl ViewHistory {
    pub fn default(entry_id: i64) -> ViewHistory {
        ViewHistory {
            entry_id: entry_id,
            last_viewed_date: None,
            video_history: None,
        }
    }
}

const ALL_COLUMN_NAMES: &[&'static str] = &["entry_id", "last_viewed_date", "farthest_ts", "farthest_ts_ratio", "farthest_ts_date"];

fn row_to_view_history(row: &Row) -> Result<ViewHistory> {
    let farthest_ts: Option<i64> = row.get(2)?;
    let video_history = if farthest_ts.is_some() {
        Some(VideoHistory {
            farthest_ts: farthest_ts.unwrap(),
            farthest_ts_ratio: row.get(3)?,
            farthest_ts_date: row.get(4)?,
        })
    } else {
        None
    };

    Ok(ViewHistory {
        entry_id: row.get(0)?,
        last_viewed_date: row.get(1)?,
        video_history,
    })
}

pub struct HistoryDb {
    conn: Connection,
}

impl HistoryDb {
    fn new_internal(conn: Connection) -> HistoryDb {
        // Create table if doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ViewHistory (
                entry_id INTEGER PRIMARY KEY,
                last_viewed_date DATETIME,
                farthest_ts INT64,
                farthest_ts_ratio FLOAT,
                farthest_ts_date DATETIME
            )",
            [],
        ).unwrap();

        HistoryDb {
            conn,
        }
    }

    pub fn new_in_memory() -> HistoryDb {
        HistoryDb::new_internal(Connection::open_in_memory().unwrap())
    }

    pub fn new(db_path: &Path) -> HistoryDb {
        let conn = Connection::open(db_path).unwrap();
        HistoryDb::new_internal(conn)
    }

    pub fn get(&self, entry_id: i64) -> Result<ViewHistory> {
        self.conn.query_row(
            &format!("SELECT {} FROM ViewHistory
                      WHERE entry_id = ?1", ALL_COLUMN_NAMES.join(",")),
            params![entry_id],
            row_to_view_history,
        ).optional()
            .map(|opt| opt.unwrap_or(ViewHistory::default(entry_id)))
    }

    pub fn mark_viewed(&mut self, entry_id: i64, video_info: Option<(i64, f32)>) -> Result<()> {
        let tx = self.conn.transaction()?;

        tx.execute(
            "INSERT OR IGNORE INTO ViewHistory (entry_id)
            VALUES (?1)",
            params![entry_id],
        )?;

        let existing = tx.query_row(
            &format!("SELECT {} FROM ViewHistory
                      WHERE entry_id = ?1", ALL_COLUMN_NAMES.join(",")),
            params![entry_id],
            row_to_view_history,
        )?;

        let rows_updated = tx.execute("UPDATE OR FAIL ViewHistory
            SET last_viewed_date = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
            WHERE entry_id = ?1", params![entry_id])?;
        assert!(rows_updated == 1);

        if let Some((ts, ts_ratio)) = video_info {
            if existing.video_history.is_none() || ts >= existing.video_history.unwrap().farthest_ts {
                tx.execute("UPDATE OR FAIL ViewHistory SET
                        farthest_ts = ?2,
                        farthest_ts_ratio = ?3,
                        farthest_ts_date = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                    WHERE entry_id = ?1", params![entry_id, ts, ts_ratio])?;
            }
        }

        tx.commit()
    }

    pub fn clear_history(&mut self, entry_id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM ViewHistory
            WHERE entry_id = ?1", params![entry_id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time;

    #[test]
    fn test_get_uninit() -> Result<()> {
        let db = HistoryDb::new_in_memory();
        let hist = db.get(42)?;
        assert_eq!(hist.entry_id, 42);
        assert!(hist.last_viewed_date.is_none());
        assert!(hist.video_history.is_none());

        Ok(())
    }

    #[test]
    fn test_mark_viewed_video() -> Result<()> {
        let mut db = HistoryDb::new_in_memory();
        db.mark_viewed(42, Some((2, 0.45)))?;

        let hist = db.get(42)?;
        println!("{:?}", hist);
        assert!(hist.last_viewed_date.is_some());
        let video_hist = hist.video_history.unwrap();
        assert_eq!(video_hist.farthest_ts, 2);
        assert_eq!(video_hist.farthest_ts_ratio, 0.45);

        thread::sleep(time::Duration::from_secs(2));

        // video_info should only preserve largest values
        db.mark_viewed(42, Some((1, 0.33)))?;
        let hist2 = db.get(42)?;
        assert_ne!(hist.last_viewed_date, hist2.last_viewed_date);
        let video_hist2 = hist2.video_history.unwrap();
        assert_eq!(video_hist2.farthest_ts, 2);
        assert_eq!(video_hist2.farthest_ts_ratio, 0.45);

        Ok(())
    }

    #[test]
    fn test_clear_history() -> Result<()> {
        let mut db = HistoryDb::new_in_memory();
        db.mark_viewed(42, Some((2, 0.45)))?;
        db.clear_history(42)?;

        let hist = db.get(42)?;
        assert!(hist.last_viewed_date.is_none());
        assert!(hist.video_history.is_none());

        // Clear should be idempotent
        db.clear_history(42)?;

        Ok(())
    }

    #[test]
    fn test_non_video() -> Result<()> {
        let mut db = HistoryDb::new_in_memory();
        db.mark_viewed(42, None)?;

        let hist = db.get(42)?;
        assert!(hist.last_viewed_date.is_some());
        assert!(hist.video_history.is_none());
        Ok(())
    }
}
