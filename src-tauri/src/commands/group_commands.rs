use crate::error::AppResult;
use crate::models::*;
use crate::repositories::env_group_repo;
use crate::repositories::history_repo;
use crate::services::env_service::EnvService;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_all_groups(state: State<'_, AppState>) -> AppResult<Vec<EnvGroup>> {
    state.with_db(|conn| env_group_repo::EnvGroupRepository::get_all(conn).map_err(Into::into))
}

#[tauri::command]
pub async fn create_group(
    state: State<'_, AppState>,
    name: String,
    description: String,
    variables: Vec<EnvVariable>,
    chain_id: Option<String>,
) -> AppResult<EnvGroup> {
    let now = chrono::Utc::now().timestamp();
    let group = EnvGroup {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        description,
        variables,
        is_active: false,
        chain_id,
        created_at: now,
        updated_at: now,
    };

    state.with_db(|conn| {
        env_group_repo::EnvGroupRepository::insert(conn, &group)?;
        history_repo::HistoryRepository::insert(
            conn,
            &HistoryRecord {
                id: uuid::Uuid::new_v4().to_string(),
                action_type: "create".to_string(),
                target_type: "group".to_string(),
                target_id: group.id.clone(),
                before_data: None,
                after_data: Some(serde_json::to_string(&group)?),
                timestamp: now,
            },
        )?;
        Ok::<_, crate::error::AppError>(())
    })?;

    Ok(group)
}

#[tauri::command]
pub async fn update_group(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    description: Option<String>,
    variables: Option<Vec<EnvVariable>>,
    chain_id: Option<Option<String>>,
) -> AppResult<EnvGroup> {
    let now = chrono::Utc::now().timestamp();

    let updated = state.with_db(|conn| -> AppResult<EnvGroup> {
        let mut group = env_group_repo::EnvGroupRepository::get_by_id(conn, &id)?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("变量组 {} 不存在", id)))?;
        let before = serde_json::to_string(&group)?;

        if let Some(n) = name { group.name = n; }
        if let Some(d) = description { group.description = d; }
        if let Some(v) = variables { group.variables = v; }
        if let Some(c) = chain_id { group.chain_id = c; }
        group.updated_at = now;

        env_group_repo::EnvGroupRepository::update(conn, &group)?;
        history_repo::HistoryRepository::insert(
            conn,
            &HistoryRecord {
                id: uuid::Uuid::new_v4().to_string(),
                action_type: "edit".to_string(),
                target_type: "group".to_string(),
                target_id: id.clone(),
                before_data: Some(before),
                after_data: Some(serde_json::to_string(&group)?),
                timestamp: now,
            },
        )?;
        Ok(group)
    })?;

    Ok(updated)
}

#[tauri::command]
pub async fn delete_group(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp();

    let (group, before_json) = state.with_db(|conn| -> AppResult<(EnvGroup, String)> {
        let group = env_group_repo::EnvGroupRepository::get_by_id(conn, &id)?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("变量组 {} 不存在", id)))?;
        let before = serde_json::to_string(&group)?;
        Ok((group, before))
    })?;

    // 如果已激活，需要先从系统中移除
    if group.is_active {
        EnvService::deactivate_group(&state, &id).await?;
    }

    state.with_db(|conn| {
        env_group_repo::EnvGroupRepository::delete(conn, &id)?;
        history_repo::HistoryRepository::insert(
            conn,
            &HistoryRecord {
                id: uuid::Uuid::new_v4().to_string(),
                action_type: "delete".to_string(),
                target_type: "group".to_string(),
                target_id: id,
                before_data: Some(before_json),
                after_data: None,
                timestamp: now,
            },
        )?;
        Ok::<_, crate::error::AppError>(())
    })?;

    Ok(())
}

#[tauri::command]
pub async fn activate_group(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<ActivationResult> {
    EnvService::activate_group(&state, &id).await
}

#[tauri::command]
pub async fn deactivate_group(state: State<'_, AppState>, id: String) -> AppResult<()> {
    EnvService::deactivate_group(&state, &id).await
}
