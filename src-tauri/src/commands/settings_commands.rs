use crate::error::AppResult;
use crate::models::*;
use crate::state::AppState;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn get_app_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
    let result = state.with_db(|conn| {
        let stmt = conn.query_row(
            "SELECT value FROM app_settings WHERE key = ?",
            ["app-settings"],
            |row| row.get::<_, String>(0),
        );
        Ok::<_, crate::error::AppError>(stmt.ok())
    })?;

    match result {
        Some(value) => {
            let settings: AppSettings = serde_json::from_str(&value)?;
            Ok(settings)
        }
        None => Ok(AppSettings::default()),
    }
}

#[tauri::command]
pub async fn set_app_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> AppResult<()> {
    state.with_db(|conn| {
        let value = serde_json::to_string(&settings)?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params!["app-settings", value, now],
        )?;
        Ok::<_, crate::error::AppError>(())
    })?;
    Ok(())
}
