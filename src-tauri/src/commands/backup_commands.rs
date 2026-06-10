use crate::error::AppResult;
use crate::models::*;
use crate::repositories::backup_repo;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn create_backup(
    state: State<'_, AppState>,
    name: String,
    scope: String,
) -> AppResult<Backup> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let platform = state.platform.clone();

    // 读取用户与系统变量，仅做备份快照
    let user_vars = platform.get_all_variables(false).await?;
    let system_vars = platform.get_all_variables(true).await?;
    let env_snapshot =
        serde_json::json!({ "user_vars": user_vars, "system_vars": system_vars }).to_string();

    let db_snapshot = state.with_db(|conn| -> AppResult<String> {
        let groups = backup_repo::BackupRepository::get_all(conn)?;
        let _ = groups; // 这里的 snapshot 以备份表名占位
        Ok("".to_string())
    })?;
    let _ = db_snapshot;

    let backup = Backup {
        id: id.clone(),
        name,
        scope,
        db_snapshot: "".to_string(),
        env_snapshot,
        created_at: now,
    };

    state.with_db(|conn| {
        backup_repo::BackupRepository::insert(conn, &backup)?;
        Ok::<_, crate::error::AppError>(())
    })?;

    Ok(backup)
}

#[tauri::command]
pub async fn get_all_backups(state: State<'_, AppState>) -> AppResult<Vec<Backup>> {
    state.with_db(|conn| backup_repo::BackupRepository::get_all(conn).map_err(Into::into))
}

#[tauri::command]
pub async fn restore_backup(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let backup = state.with_db(|conn| -> AppResult<Backup> {
        backup_repo::BackupRepository::get_by_id(conn, &id)?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("备份 {} 不存在", id)))
    })?;

    let _ = backup; // 恢复逻辑可按业务需求扩展；这里保留接口占位
    Ok(())
}

#[tauri::command]
pub async fn delete_backup(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.with_db(|conn| {
        backup_repo::BackupRepository::delete(conn, &id)?;
        Ok::<_, crate::error::AppError>(())
    })
}

#[tauri::command]
pub async fn export_backup(
    state: State<'_, AppState>,
    id: String,
    path: String,
) -> AppResult<()> {
    let backup = state.with_db(|conn| -> AppResult<Backup> {
        backup_repo::BackupRepository::get_by_id(conn, &id)?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("备份 {} 不存在", id)))
    })?;

    let export_data = serde_json::json!({
        "version": "2.0",
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "backup": backup
    });

    std::fs::write(&path, export_data.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn import_backup(state: State<'_, AppState>, path: String) -> AppResult<Backup> {
    let content = std::fs::read_to_string(&path)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;
    let backup: Backup = serde_json::from_value(data["backup"].clone())?;

    state.with_db(|conn| {
        backup_repo::BackupRepository::insert(conn, &backup)?;
        Ok::<_, crate::error::AppError>(())
    })?;

    Ok(backup)
}
