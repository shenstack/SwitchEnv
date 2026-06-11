use crate::models::EnvGroup;
use rusqlite::{params, Connection};

pub struct EnvGroupRepository;

impl EnvGroupRepository {
    pub fn get_all(conn: &mut Connection) -> Result<Vec<EnvGroup>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, variables, is_active, created_at, updated_at FROM env_groups ORDER BY updated_at DESC"
        )?;

        let groups = stmt.query_map([], |row| {
            Ok(EnvGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                variables: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                is_active: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(groups)
    }

    pub fn get_by_id(conn: &mut Connection, id: &str) -> Result<Option<EnvGroup>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, variables, is_active, created_at, updated_at FROM env_groups WHERE id = ?"
        )?;

        let mut rows = stmt.query_map([id], |row| {
            Ok(EnvGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                variables: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                is_active: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        Ok(rows.next().transpose()?)
    }

    pub fn insert(conn: &mut Connection, group: &EnvGroup) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO env_groups (id, name, description, variables, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                group.id,
                group.name,
                group.description,
                serde_json::to_string(&group.variables).unwrap_or_default(),
                group.is_active as i32,
                group.created_at,
                group.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn update(conn: &mut Connection, group: &EnvGroup) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE env_groups SET name = ?2, description = ?3, variables = ?4, is_active = ?5, updated_at = ?6 WHERE id = ?1",
            params![
                group.id,
                group.name,
                group.description,
                serde_json::to_string(&group.variables).unwrap_or_default(),
                group.is_active as i32,
                group.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn delete(conn: &mut Connection, id: &str) -> Result<(), rusqlite::Error> {
        conn.execute("DELETE FROM env_groups WHERE id = ?", [id])?;
        Ok(())
    }
}
