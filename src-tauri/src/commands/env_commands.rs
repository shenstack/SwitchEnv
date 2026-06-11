use crate::error::AppResult;
use crate::models::*;
use crate::state::AppState;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn get_all_env_vars(
    state: State<'_, AppState>,
    is_system: bool,
) -> AppResult<Vec<EnvVar>> {
    state
        .platform
        .get_all_variables(is_system)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_env_var(state: State<'_, AppState>, name: String) -> AppResult<Option<String>> {
    state.platform.get_variable(&name).await.map_err(Into::into)
}

#[tauri::command]
pub async fn set_env_var(
    state: State<'_, AppState>,
    name: String,
    value: String,
    is_system: bool,
) -> AppResult<()> {
    state
        .platform
        .set_variable(&name, &value, is_system)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn remove_env_var(
    state: State<'_, AppState>,
    name: String,
    is_system: bool,
) -> AppResult<()> {
    state
        .platform
        .remove_variable(&name, is_system)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn can_modify_system(state: State<'_, AppState>) -> AppResult<bool> {
    state.platform.can_modify_system().await.map_err(Into::into)
}

#[tauri::command]
pub async fn refresh_environment(state: State<'_, AppState>) -> AppResult<()> {
    state.platform.refresh_environment().await.map_err(Into::into)
}

#[tauri::command]
pub async fn open_system_settings(state: State<'_, AppState>) -> AppResult<()> {
    state.platform.open_system_settings().await.map_err(Into::into)
}

#[tauri::command]
pub async fn get_shell_config_info(state: State<'_, AppState>) -> AppResult<ShellConfigInfo> {
    state.platform.get_shell_config_info().await.map_err(Into::into)
}

/// 导出用户级或系统级环境变量为 key=value 格式的文本（每行一个变量）。
/// save_path 提供时写入文件并返回文件绝对路径；
/// save_path 缺省时直接返回文本内容，由前端自行处理保存。
#[tauri::command]
pub async fn export_env_vars(
    state: State<'_, AppState>,
    is_system: bool,
    save_path: Option<PathBuf>,
) -> AppResult<String> {
    let vars = state.platform.get_all_variables(is_system).await?;

    // 按变量名排序以提高可读性；key=value 格式保证无多余空行。
    let mut sorted = vars;
    sorted.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let content = sorted
        .iter()
        .map(|v| format!("{}={}", v.name, v.value))
        .collect::<Vec<_>>()
        .join("\n");

    if let Some(path) = save_path {
        std::fs::write(&path, &content)?;
        let abs = std::fs::canonicalize(&path).unwrap_or(path);
        return Ok(abs.to_string_lossy().to_string());
    }
    Ok(content)
}
