CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS env_groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    variables TEXT NOT NULL DEFAULT '[]',
    is_active INTEGER NOT NULL DEFAULT 0,
    chain_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS trash_history (
    id TEXT PRIMARY KEY,
    action_type TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    before_data TEXT,
    after_data TEXT,
    timestamp INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS backups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    scope TEXT NOT NULL,
    db_snapshot TEXT NOT NULL,
    env_snapshot TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_env_groups_chain ON env_groups(chain_id);
CREATE INDEX IF NOT EXISTS idx_env_groups_active ON env_groups(is_active);
CREATE INDEX IF NOT EXISTS idx_trash_history_type ON trash_history(target_type);
CREATE INDEX IF NOT EXISTS idx_trash_history_timestamp ON trash_history(timestamp);
