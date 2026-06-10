use crate::models::Template;
use rusqlite::{params, Connection};

/// 模板的持久化访问层。
pub struct TemplateRepository;

impl TemplateRepository {
    /// 获取所有模板，按更新时间倒序。
    pub fn get_all(conn: &mut Connection) -> Result<Vec<Template>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, keys, created_at, updated_at FROM templates ORDER BY updated_at DESC",
        )?;

        let list = stmt
            .query_map([], |row| {
                Ok(Template {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    keys: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(list)
    }

    /// 通过 ID 获取模板。
    pub fn get_by_id(conn: &mut Connection, id: &str) -> Result<Option<Template>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, keys, created_at, updated_at FROM templates WHERE id = ?",
        )?;

        let mut rows = stmt.query_map([id], |row| {
            Ok(Template {
                id: row.get(0)?,
                name: row.get(1)?,
                keys: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        Ok(rows.next().transpose()?)
    }

    /// 插入新模板。
    pub fn insert(conn: &mut Connection, tpl: &Template) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO templates (id, name, keys, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                tpl.id,
                tpl.name,
                serde_json::to_string(&tpl.keys).unwrap_or_default(),
                tpl.created_at,
                tpl.updated_at,
            ],
        )?;
        Ok(())
    }

    /// 更新模板。
    pub fn update(conn: &mut Connection, tpl: &Template) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE templates SET name = ?2, keys = ?3, updated_at = ?4 WHERE id = ?1",
            params![
                tpl.id,
                tpl.name,
                serde_json::to_string(&tpl.keys).unwrap_or_default(),
                tpl.updated_at,
            ],
        )?;
        Ok(())
    }

    /// 删除模板。
    pub fn delete(conn: &mut Connection, id: &str) -> Result<(), rusqlite::Error> {
        conn.execute("DELETE FROM templates WHERE id = ?", [id])?;
        Ok(())
    }
}
