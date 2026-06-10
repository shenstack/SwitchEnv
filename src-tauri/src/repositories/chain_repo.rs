use crate::models::Chain;
use rusqlite::{params, Connection};

/// 锁链的持久化访问层。
pub struct ChainRepository;

impl ChainRepository {
    /// 获取所有锁链，按更新时间倒序。
    pub fn get_all(conn: &mut Connection) -> Result<Vec<Chain>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, updated_at FROM chains ORDER BY updated_at DESC",
        )?;

        let list = stmt
            .query_map([], |row| {
                Ok(Chain {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(list)
    }

    /// 通过 ID 获取锁链。
    pub fn get_by_id(conn: &mut Connection, id: &str) -> Result<Option<Chain>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, updated_at FROM chains WHERE id = ?",
        )?;

        let mut rows = stmt.query_map([id], |row| {
            Ok(Chain {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;

        Ok(rows.next().transpose()?)
    }

    /// 通过名称查找锁链（用于导入时去重）。
    pub fn get_by_name(conn: &mut Connection, name: &str) -> Result<Option<Chain>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, updated_at FROM chains WHERE name = ?",
        )?;

        let mut rows = stmt.query_map([name], |row| {
            Ok(Chain {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;

        Ok(rows.next().transpose()?)
    }

    /// 插入新锁链。
    pub fn insert(conn: &mut Connection, chain: &Chain) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO chains (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![chain.id, chain.name, chain.created_at, chain.updated_at],
        )?;
        Ok(())
    }

    /// 更新锁链。
    pub fn update(conn: &mut Connection, chain: &Chain) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE chains SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![chain.id, chain.name, chain.updated_at],
        )?;
        Ok(())
    }

    /// 删除锁链，并将所有组的 chain_id 清空。
    pub fn delete(conn: &mut Connection, id: &str) -> Result<(), rusqlite::Error> {
        // 将引用该 chain 的 env_groups 的 chain_id 置空
        conn.execute("UPDATE env_groups SET chain_id = NULL WHERE chain_id = ?", [id])?;
        conn.execute("DELETE FROM chains WHERE id = ?", [id])?;
        Ok(())
    }
}
