use crate::error::AppResult;
use crate::models::*;
use crate::state::AppState;
use rusqlite::params;
use std::fs;
use std::path::Path;
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

/// 清理超过保留天数的日志文件，返回被删除的文件数。
#[tauri::command]
pub async fn cleanup_logs(state: State<'_, AppState>, retention_days: i32) -> AppResult<i32> {
    let log_path = state.log_path.clone();
    let logs_dir = log_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| log_path.clone());

    let now = chrono::Utc::now().naive_utc().date();
    let mut deleted = 0;

    if logs_dir.exists() {
        if let Ok(entries) = fs::read_dir(&logs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                if !file_name.ends_with(".log") {
                    continue;
                }
                let date_str = file_name.trim_end_matches(".log");
                if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    let diff = (now - date).num_days();
                    if diff > retention_days as i64 {
                        if fs::remove_file(&path).is_ok() {
                            deleted += 1;
                        }
                    }
                }
            }
        }
    }

    log::info!(
        "[cleanup_logs] 日志清理完成, 保留天数={}, 删除文件数={}, 目录={}",
        retention_days,
        deleted,
        logs_dir.display()
    );
    Ok(deleted)
}

/// 在应用启动时根据设置自动清理过期日志。
pub fn run_startup_cleanup(state: &AppState) -> AppResult<i32> {
    let settings = state.with_db(|conn| {
        let stmt = conn.query_row(
            "SELECT value FROM app_settings WHERE key = ?",
            ["app-settings"],
            |row| row.get::<_, String>(0),
        );
        Ok::<_, crate::error::AppError>(stmt.ok())
    })?;

    let settings: AppSettings = match settings {
        Some(v) => serde_json::from_str(&v).unwrap_or_default(),
        None => AppSettings::default(),
    };

    if !settings.logs.auto_cleanup {
        return Ok(0);
    }

    let retention_days = settings.logs.retention_days;
    let log_path = state.log_path.clone();
    let logs_dir = log_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| log_path.clone());

    let now = chrono::Utc::now().naive_utc().date();
    let mut deleted = 0;

    if logs_dir.exists() {
        if let Ok(entries) = fs::read_dir(&logs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                if !file_name.ends_with(".log") {
                    continue;
                }
                let date_str = file_name.trim_end_matches(".log");
                if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    let diff = (now - date).num_days();
                    if diff > retention_days as i64 {
                        if fs::remove_file(&path).is_ok() {
                            deleted += 1;
                        }
                    }
                }
            }
        }
    }

    log::info!(
        "[startup_cleanup] 日志清理完成, 保留天数={}, 删除文件数={}",
        retention_days,
        deleted
    );
    Ok(deleted)
}
