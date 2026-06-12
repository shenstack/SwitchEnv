use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
    #[serde(rename = "isSystem")]
    pub is_system: bool,
    #[serde(rename = "isReadonly")]
    pub is_readonly: bool,
    #[serde(rename = "source", skip_serializing_if = "Option::is_none", default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVariable {
    pub name: String,
    pub value: String,
    #[serde(rename = "isHidden")]
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub variables: Vec<EnvVariable>,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupInput {
    pub name: String,
    pub description: String,
    pub variables: Vec<EnvVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub variables: Option<Vec<EnvVariable>>,
}

/// 激活变量组的结果：
/// - conflicts：与系统变量或其他已激活组存在冲突时填充，
///   success=false 时表示有冲突尚未被强制覆盖。
/// - errors：写入系统时发生的不可恢复错误。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationResult {
    pub success: bool,
    pub conflicts: Vec<EnvVarConflict>,
    pub errors: Vec<String>,
}

/// 冲突条目：记录与系统注册表或其他已激活变量组的同名变量差异。
/// source="system" 表示来自用户环境变量注册表；
/// 其他 source 为具体变量组 id，此时 source_group_name 会附带组名。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarConflict {
    pub name: String,
    #[serde(rename = "existingValue")]
    pub existing_value: String,
    #[serde(rename = "newValue")]
    pub new_value: String,
    pub source: String,
    #[serde(rename = "sourceGroupName")]
    pub source_group_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfigInfo {
    #[serde(rename = "shellPath")]
    pub shell_path: String,
    #[serde(rename = "configFile")]
    pub config_file: String,
    #[serde(rename = "managedVars")]
    pub managed_vars: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: String,
    #[serde(rename = "actionType")]
    pub action_type: String,
    #[serde(rename = "targetType")]
    pub target_type: String,
    #[serde(rename = "targetId")]
    pub target_id: String,
    #[serde(rename = "beforeData")]
    pub before_data: Option<String>,
    #[serde(rename = "afterData")]
    pub after_data: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub id: String,
    pub name: String,
    pub scope: String,
    #[serde(rename = "dbSnapshot")]
    pub db_snapshot: String,
    #[serde(rename = "envSnapshot")]
    pub env_snapshot: String,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: ThemeSettings,
    pub history: HistorySettings,
    pub logs: LogSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub mode: String,
    #[serde(rename = "fontLevel")]
    pub font_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySettings {
    #[serde(rename = "autoCleanup")]
    pub auto_cleanup: bool,
    #[serde(rename = "retentionDays")]
    pub retention_days: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSettings {
    #[serde(rename = "autoCleanup")]
    pub auto_cleanup: bool,
    #[serde(rename = "retentionDays")]
    pub retention_days: i32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: ThemeSettings {
                mode: "system".to_string(),
                font_level: 2,
            },
            history: HistorySettings {
                auto_cleanup: true,
                retention_days: 30,
            },
            logs: LogSettings {
                auto_cleanup: true,
                retention_days: 3,
            },
        }
    }
}

/// 变量组模板：保存一组常用的变量名，快速创建变量组。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub keys: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

/// 变量组导入导出的 JSON 载体。
/// 向后兼容：旧文件中若存在 chains / groups[i].chainId，
/// 在反序列化时会被忽略（借助 serde 未知字段默认丢弃策略 + deny_unknown_fields=false）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupExportPayload {
    #[serde(rename = "type")]
    pub type_: String,
    pub version: i32,
    #[serde(rename = "exportedAt")]
    pub exported_at: i64,
    pub groups: Vec<GroupExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupExport {
    pub name: String,
    pub description: String,
    pub variables: Vec<EnvVariable>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<i64>,
}

/// 变量差异项，用于冲突组中逐变量对比。
/// diff_type 取值: "added_only_incoming" | "missing_only_existing" | "value_changed" | "identical"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportVarDiff {
    pub name: String,
    #[serde(rename = "diffType")]
    pub diff_type: String,
    #[serde(rename = "existingValue", skip_serializing_if = "Option::is_none", default)]
    pub existing_value: Option<String>,
    #[serde(rename = "incomingValue", skip_serializing_if = "Option::is_none", default)]
    pub incoming_value: Option<String>,
    #[serde(rename = "existingIsHidden", skip_serializing_if = "Option::is_none", default)]
    pub existing_is_hidden: Option<bool>,
    #[serde(rename = "incomingIsHidden", skip_serializing_if = "Option::is_none", default)]
    pub incoming_is_hidden: Option<bool>,
}

/// 单个冲突组，用于前端弹窗展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConflictGroup {
    pub name: String,
    #[serde(rename = "existingDescription")]
    pub existing_description: String,
    #[serde(rename = "incomingDescription")]
    pub incoming_description: String,
    #[serde(rename = "varDiffs")]
    pub var_diffs: Vec<ImportVarDiff>,
    #[serde(rename = "isIdentical")]
    pub is_identical: bool,
}

/// 预检返回的整体结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreviewResult {
    #[serde(rename = "newGroups")]
    pub new_groups: Vec<GroupExport>,
    #[serde(rename = "conflictGroups")]
    pub conflict_groups: Vec<ImportConflictGroup>,
}
