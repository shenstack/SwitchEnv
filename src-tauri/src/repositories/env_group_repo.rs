use crate::models::EnvGroup;
use rusqlite::{params, Connection};

pub struct EnvGroupRepository;

impl EnvGroupRepository {
    pub fn get_all(conn: &mut Connection) -> Result<Vec<EnvGroup>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, variables, is_active, chain_id, created_at, updated_at FROM env_groups ORDER BY updated_at DESC"
        )?;

        let groups = stmt.query_map([], |row| {
            Ok(EnvGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                variables: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                is_active: row.get::<_, i32>(4)? != 0,
                chain_id: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(groups)
    }

    pub fn get_by_id(conn: &mut Connection, id: &str) -> Result<Option<EnvGroup>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, variables, is_active, chain_id, created_at, updated_at FROM env_groups WHERE id = ?"
        )?;

        let mut rows = stmt.query_map([id], |row| {
            Ok(EnvGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                variables: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                is_active: row.get::<_, i32>(4)? != 0,
                chain_id: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        Ok(rows.next().transpose()?)
    }

    pub fn insert(conn: &mut Connection, group: &EnvGroup) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO env_groups (id, name, description, variables, is_active, chain_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                group.id,
                group.name,
                group.description,
                serde_json::to_string(&group.variables).unwrap_or_default(),
                group.is_active as i32,
                group.chain_id,
                group.created_at,
                group.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn update(conn: &mut Connection, group: &EnvGroup) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE env_groups SET name = ?2, description = ?3, variables = ?4, is_active = ?5, chain_id = ?6, updated_at = ?7 WHERE id = ?1",
            params![
                group.id,
                group.name,
                group.description,
                serde_json::to_string(&group.variables).unwrap_or_default(),
                group.is_active as i32,
                group.chain_id,
                group.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn delete(conn: &mut Connection, id: &str) -> Result<(), rusqlite::Error> {
        conn.execute("DELETE FROM env_groups WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn get_active_by_chain(conn: &mut Connection, chain_id: &str) -> Result<Vec<EnvGroup>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, variables, is_active, chain_id, created_at, updated_at FROM env_groups WHERE chain_id = ?1 AND is_active = 1"
        )?;

        let groups = stmt.query_map([chain_id], |row| {
            Ok(EnvGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                variables: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                is_active: row.get::<_, i32>(4)? != 0,
                chain_id: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(groups)
    }
}
