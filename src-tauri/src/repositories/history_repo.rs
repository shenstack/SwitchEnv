use crate::models::HistoryRecord;
use rusqlite::{params, Connection};

pub struct HistoryRepository;

impl HistoryRepository {
    pub fn get_all(
        conn: &mut Connection,
        target_type: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<HistoryRecord>, rusqlite::Error> {
        let limit_val = limit.unwrap_or(100);
        let (sql, has_filter) = match target_type {
            Some(_) => (
                "SELECT id, action_type, target_type, target_id, before_data, after_data, \
                 timestamp FROM trash_history WHERE target_type = ?1 \
                 ORDER BY timestamp DESC LIMIT ?2",
                true,
            ),
            None => (
                "SELECT id, action_type, target_type, target_id, before_data, after_data, \
                 timestamp FROM trash_history ORDER BY timestamp DESC LIMIT ?1",
                false,
            ),
        };

        let mut stmt = conn.prepare(sql)?;
        let mapped = if has_filter {
            stmt.query_map(params![target_type, limit_val], map_row)?
        } else {
            stmt.query_map(params![limit_val], map_row)?
        };

        let mut out = Vec::new();
        for row in mapped {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn insert(conn: &mut Connection, record: &HistoryRecord) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO trash_history (id, action_type, target_type, target_id, before_data, \
             after_data, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.id,
                record.action_type,
                record.target_type,
                record.target_id,
                record.before_data,
                record.after_data,
                record.timestamp
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(
        conn: &mut Connection,
        id: &str,
    ) -> Result<Option<HistoryRecord>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, action_type, target_type, target_id, before_data, after_data, \
             timestamp FROM trash_history WHERE id = ?",
        )?;
        let mut rows = stmt.query_map([id], map_row)?;
        Ok(rows.next().transpose()?)
    }

    pub fn delete(conn: &mut Connection, id: &str) -> Result<(), rusqlite::Error> {
        conn.execute("DELETE FROM trash_history WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn clear_by_type(
        conn: &mut Connection,
        target_type: Option<&str>,
    ) -> Result<usize, rusqlite::Error> {
        match target_type {
            Some(t) => conn.execute("DELETE FROM trash_history WHERE target_type = ?", [t]),
            None => conn.execute("DELETE FROM trash_history", []),
        }
    }
}

/// 共享的行映射函数 — 避免 match arms 中产生类型不同的匿名 closure。
fn map_row(row: &rusqlite::Row<'_>) -> Result<HistoryRecord, rusqlite::Error> {
    Ok(HistoryRecord {
        id: row.get(0)?,
        action_type: row.get(1)?,
        target_type: row.get(2)?,
        target_id: row.get(3)?,
        before_data: row.get(4)?,
        after_data: row.get(5)?,
        timestamp: row.get(6)?,
    })
}
