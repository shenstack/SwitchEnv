use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
    #[serde(rename = "isSystem")]
    pub is_system: bool,
    #[serde(rename = "isReadonly")]
    pub is_readonly: bool,
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
    #[serde(rename = "chainId")]
    pub chain_id: Option<String>,
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
    #[serde(rename = "chainId")]
    pub chain_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub variables: Option<Vec<EnvVariable>>,
    #[serde(rename = "chainId")]
    pub chain_id: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationResult {
    pub success: bool,
    pub conflicts: Vec<EnvVarConflict>,
    pub deactivated_groups: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarConflict {
    pub name: String,
    #[serde(rename = "existingValue")]
    pub existing_value: String,
    #[serde(rename = "newValue")]
    pub new_value: String,
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
    pub notification: NotificationSettings,
    pub history: HistorySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub mode: String,
    #[serde(rename = "fontLevel")]
    pub font_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    #[serde(rename = "desktopEnabled")]
    pub desktop_enabled: bool,
    #[serde(rename = "inAppEnabled")]
    pub in_app_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySettings {
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
            notification: NotificationSettings {
                desktop_enabled: true,
                in_app_enabled: true,
            },
            history: HistorySettings {
                auto_cleanup: true,
                retention_days: 30,
            },
        }
    }
}
