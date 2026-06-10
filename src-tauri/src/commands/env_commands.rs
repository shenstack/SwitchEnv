use crate::error::AppResult;
use crate::models::*;
use crate::state::AppState;
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
