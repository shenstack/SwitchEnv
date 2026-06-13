pub mod macos;
pub mod shell_profile;

#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::models::{EnvVar, ShellConfigInfo};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 平台信息（操作系统 / 架构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("权限不足: {0}")]
    #[allow(dead_code)]
    PermissionDenied(String),
    #[error("注册表操作失败: {0}")]
    #[allow(dead_code)]
    RegistryError(String),
    #[error("Shell 配置操作失败: {0}")]
    ShellConfigError(String),
    #[error("值长度超过限制: {0}")]
    #[allow(dead_code)]
    ValueTooLong(String),
    #[error("平台不支持: {0}")]
    #[allow(dead_code)]
    Unsupported(String),
    #[error("命令执行失败: {0}")]
    CommandFailed(String),
}

/// `shell_profile.rs` 的方法用 `Result<_, String>` 表达错误，这里统一转换。
impl From<String> for PlatformError {
    fn from(msg: String) -> Self {
        PlatformError::ShellConfigError(msg)
    }
}

#[async_trait]
pub trait PlatformService: Send + Sync {
    async fn get_all_variables(&self, is_system_scope: bool) -> Result<Vec<EnvVar>, PlatformError>;
    async fn set_variable(&self, name: &str, value: &str, is_system_scope: bool) -> Result<(), PlatformError>;
    async fn remove_variable(&self, name: &str, is_system_scope: bool) -> Result<(), PlatformError>;
    async fn get_variable(&self, name: &str) -> Result<Option<String>, PlatformError>;
    async fn can_modify_system(&self) -> Result<bool, PlatformError>;
    async fn refresh_environment(&self) -> Result<(), PlatformError>;
    #[allow(dead_code)]
    fn get_value_length_limit(&self) -> usize;
    async fn open_system_settings(&self) -> Result<(), PlatformError>;
    async fn get_shell_config_info(&self) -> Result<ShellConfigInfo, PlatformError>;
    #[allow(dead_code)]
    fn get_platform_info(&self) -> PlatformInfo;
}

/// 构造一个与当前操作系统对应的 PlatformService 实现。
pub fn create_platform_service() -> Box<dyn PlatformService> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatformService)
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSPlatformService::new())
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxPlatformService::new())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        compile_error!("Unsupported platform")
    }
}
