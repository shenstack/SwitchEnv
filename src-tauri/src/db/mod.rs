pub mod migrations;

use crate::error::{AppError, AppResult};
use rusqlite::Connection;

/// 运行数据库迁移（从 `migrations.rs` 读取 `Migrations`）。
pub fn run_migrations(conn: &mut Connection) -> AppResult<()> {
    migrations::get_migrations()
        .to_latest(conn)
        .map_err(|e| AppError::Migration(e.to_string()))?;
    Ok(())
}
