use crate::error::AppResult;
use crate::models::Chain;
use crate::repositories::{chain_repo, env_group_repo};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_all_chains(state: State<'_, AppState>) -> AppResult<Vec<Chain>> {
    state.with_db(|conn| chain_repo::ChainRepository::get_all(conn).map_err(Into::into))
}

#[tauri::command]
pub async fn create_chain(state: State<'_, AppState>, name: String) -> AppResult<Chain> {
    if name.trim().is_empty() {
        return Err(crate::error::AppError::Validation(
            "锁链名称不能为空".to_string(),
        ));
    }
    let now = chrono::Utc::now().timestamp();
    let chain = Chain {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        created_at: now,
        updated_at: now,
    };

    state.with_db(|conn| {
        chain_repo::ChainRepository::insert(conn, &chain)?;
        Ok::<_, crate::error::AppError>(())
    })?;

    Ok(chain)
}

#[tauri::command]
pub async fn update_chain(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> AppResult<Chain> {
    if name.trim().is_empty() {
        return Err(crate::error::AppError::Validation(
            "锁链名称不能为空".to_string(),
        ));
    }
    let now = chrono::Utc::now().timestamp();

    let updated = state.with_db(|conn| -> AppResult<Chain> {
        let existing = chain_repo::ChainRepository::get_by_id(conn, &id)?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("锁链 {} 不存在", id)))?;
        let chain = Chain {
            id: existing.id.clone(),
            name: name.trim().to_string(),
            created_at: existing.created_at,
            updated_at: now,
        };
        chain_repo::ChainRepository::update(conn, &chain)?;
        Ok(chain)
    })?;

    Ok(updated)
}

#[tauri::command]
pub async fn delete_chain(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.with_db(|conn| {
        chain_repo::ChainRepository::delete(conn, &id)?;
        Ok::<_, crate::error::AppError>(())
    })?;
    Ok(())
}

/// 将变量组分配到锁链（chain_id 为 None 表示移出锁链）。
#[tauri::command]
pub async fn assign_group_to_chain(
    state: State<'_, AppState>,
    group_id: String,
    chain_id: Option<String>,
) -> AppResult<()> {
    state.with_db(|conn| -> AppResult<()> {
        let mut group = env_group_repo::EnvGroupRepository::get_by_id(conn, &group_id)?
            .ok_or_else(|| {
                crate::error::AppError::NotFound(format!("变量组 {} 不存在", group_id))
            })?;
        group.chain_id = chain_id
            .clone()
            .filter(|c| !c.trim().is_empty())
            .map(|c| c.trim().to_string());
        group.updated_at = chrono::Utc::now().timestamp();
        env_group_repo::EnvGroupRepository::update(conn, &group)?;
        Ok(())
    })?;
    Ok(())
}
