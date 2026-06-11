-- 增量迁移 002：变量组模板支持
-- 新增：templates 表保存变量名模板。
-- 清理：移除旧版本可能创建的 chains 表（锁链功能已废弃）。
CREATE TABLE IF NOT EXISTS templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    keys TEXT NOT NULL DEFAULT '[]',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_templates_name ON templates(name);

DROP TABLE IF EXISTS chains;
