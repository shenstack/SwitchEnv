use rusqlite_migration::{Migrations, M};

/// 构建数据库迁移链：
/// - v1: 初始表结构（app_settings / env_groups / trash_history / backups）
/// - v2: 新增 templates（变量组模板）、chains（互斥分组锁链）
pub fn get_migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(include_str!("schema.sql")),
        M::up(include_str!("migration_002.sql")),
    ])
}
