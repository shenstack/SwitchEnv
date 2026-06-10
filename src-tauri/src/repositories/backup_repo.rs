use crate::models::Backup;
use rusqlite::{params, Connection};

pub struct BackupRepository;

impl BackupRepository {
    pub fn get_all(conn: &mut Connection) -> Result<Vec<Backup>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, scope, db_snapshot, env_snapshot, created_at FROM backups ORDER BY created_at DESC"
        )?;

        let backups = stmt.query_map([], |row| {
            Ok(Backup {
                id: row.get(0)?,
                name: row.get(1)?,
                scope: row.get(2)?,
                db_snapshot: row.get(3)?,
                env_snapshot: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(backups)
    }

    pub fn get_by_id(conn: &mut Connection, id: &str) -> Result<Option<Backup>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, scope, db_snapshot, env_snapshot, created_at FROM backups WHERE id = ?"
        )?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(Backup {
                id: row.get(0)?,
                name: row.get(1)?,
                scope: row.get(2)?,
                db_snapshot: row.get(3)?,
                env_snapshot: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn insert(conn: &mut Connection, backup: &Backup) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO backups (id, name, scope, db_snapshot, env_snapshot, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![backup.id, backup.name, backup.scope, backup.db_snapshot, backup.env_snapshot, backup.created_at],
        )?;
        Ok(())
    }

    pub fn delete(conn: &mut Connection, id: &str) -> Result<(), rusqlite::Error> {
        conn.execute("DELETE FROM backups WHERE id = ?", [id])?;
        Ok(())
    }
}
