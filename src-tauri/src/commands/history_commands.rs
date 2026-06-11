use crate::error::AppResult;
use crate::models::*;
use crate::repositories::env_group_repo;
use crate::repositories::history_repo;
use crate::services::env_service::EnvService;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_history(
    state: State<'_, AppState>,
    target_type: Option<String>,
    limit: Option<usize>,
) -> AppResult<Vec<HistoryRecord>> {
    state.with_db(|conn| {
        history_repo::HistoryRepository::get_all(conn, target_type.as_deref(), limit)
            .map_err(Into::into)
    })
}

#[tauri::command]
pub async fn restore_history(state: State<'_, AppState>, id: String) -> AppResult<()> {
    // 读取历史记录（同步）
    let record = state.with_db(|conn| -> AppResult<HistoryRecord> {
        history_repo::HistoryRepository::get_by_id(conn, &id)?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("历史记录 {} 不存在", id)))
    })?;

    match record.action_type.as_str() {
        "delete" => {
            if let Some(before) = &record.before_data {
                let group: EnvGroup = serde_json::from_str(before)?;
                state.with_db(|conn| {
                    env_group_repo::EnvGroupRepository::insert(conn, &group)?;
                    Ok::<_, crate::error::AppError>(())
                })?;
                for var in &group.variables {
                    let _ = state.platform.set_variable(&var.name, &var.value, false).await;
                }
            }
        }
        "edit" => {
            if let Some(before) = &record.before_data {
                let group: EnvGroup = serde_json::from_str(before)?;
                state.with_db(|conn| {
                    env_group_repo::EnvGroupRepository::update(conn, &group)?;
                    Ok::<_, crate::error::AppError>(())
                })?;
            }
        }
        "create" => {
            state.with_db(|conn| {
                env_group_repo::EnvGroupRepository::delete(conn, &record.target_id)?;
                Ok::<_, crate::error::AppError>(())
            })?;
        }
        "activate" => {
            let _ = EnvService::deactivate_group(&state, &record.target_id).await;
        }
        "deactivate" => {
            let _ = EnvService::activate_group(&state, &record.target_id, true).await;
        }
        _ => {}
    }

    state.with_db(|conn| {
        history_repo::HistoryRepository::delete(conn, &id)?;
        Ok::<_, crate::error::AppError>(())
    })?;
    Ok(())
}

#[tauri::command]
pub async fn clear_history(
    state: State<'_, AppState>,
    target_type: Option<String>,
) -> AppResult<()> {
    state.with_db(|conn| {
        history_repo::HistoryRepository::clear_by_type(conn, target_type.as_deref())?;
        Ok::<_, crate::error::AppError>(())
    })?;
    Ok(())
}
