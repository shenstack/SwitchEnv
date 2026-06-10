use crate::error::{AppError, AppResult};
use crate::models::{ChainExport, EnvGroup, GroupExport, GroupExportPayload, HistoryRecord};
use crate::repositories::{chain_repo, env_group_repo, history_repo};
use crate::services::env_service::EnvService;
use crate::state::AppState;
use std::path::PathBuf;
use tauri::State;

/// 导出指定变量组。
/// - 若提供 `save_path`：将 JSON 写入该文件，返回保存的绝对路径。
/// - 否则：返回生成的 JSON 字符串。
#[tauri::command]
pub async fn export_groups(
    state: State<'_, AppState>,
    group_ids: Option<Vec<String>>,
    save_path: Option<PathBuf>,
) -> AppResult<String> {
    let (chains, groups) = state.with_db(|conn| -> AppResult<(Vec<ChainExport>, Vec<EnvGroup>)> {
        let all_groups = env_group_repo::EnvGroupRepository::get_all(conn)?;
        let filtered = if let Some(ids) = &group_ids {
            all_groups
                .into_iter()
                .filter(|g| ids.contains(&g.id))
                .collect()
        } else {
            all_groups
        };
        let all_chains = chain_repo::ChainRepository::get_all(conn)?;
        let chain_ids: std::collections::HashSet<String> = filtered
            .iter()
            .filter_map(|g| g.chain_id.clone())
            .collect();
        let export_chains = all_chains
            .into_iter()
            .filter(|c| chain_ids.contains(&c.id))
            .map(|c| ChainExport {
                id: Some(c.id),
                name: c.name,
            })
            .collect();
        Ok((export_chains, filtered))
    })?;

    let now = chrono::Utc::now().timestamp();
    let payload = GroupExportPayload {
        type_: "env-groups-export".to_string(),
        version: 2,
        exported_at: now,
        chains,
        groups: groups
            .iter()
            .map(|g| GroupExport {
                name: g.name.clone(),
                description: g.description.clone(),
                variables: g.variables.clone(),
                chain_id: g.chain_id.clone(),
                created_at: Some(g.created_at),
            })
            .collect(),
    };

    let json =
        serde_json::to_string_pretty(&payload).map_err(|e| AppError::Other(e.to_string()))?;

    // 若传入保存路径，则写入文件并返回绝对路径给前端展示。
    if let Some(path) = save_path {
        std::fs::write(&path, &json).map_err(AppError::Io)?;
        let abs = std::fs::canonicalize(&path).unwrap_or(path);
        return Ok(abs.to_string_lossy().to_string());
    }
    Ok(json)
}

/// 从 JSON 字符串导入变量组。
#[tauri::command]
pub async fn import_groups(state: State<'_, AppState>, json: String) -> AppResult<i32> {
    let payload: GroupExportPayload =
        serde_json::from_str(&json).map_err(|e| AppError::Validation(format!(
            "导入的 JSON 格式不正确: {}",
            e
        )))?;
    if payload.type_ != "env-groups-export" || payload.groups.is_empty() {
        return Err(AppError::Validation(
            "文件格式不正确或未包含变量组".to_string(),
        ));
    }

    let count = state.with_db(|conn| -> AppResult<i32> {
        // 1) 处理锁链：同名复用，否则新建
        let mut chain_id_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for ce in &payload.chains {
            if let Some(existing) = chain_repo::ChainRepository::get_by_name(conn, &ce.name)? {
                if let Some(src_id) = &ce.id {
                    chain_id_map.insert(src_id.clone(), existing.id.clone());
                }
                continue;
            }
            let now = chrono::Utc::now().timestamp();
            let new_chain = crate::models::Chain {
                id: uuid::Uuid::new_v4().to_string(),
                name: ce.name.clone(),
                created_at: now,
                updated_at: now,
            };
            if let Some(src_id) = &ce.id {
                chain_id_map.insert(src_id.clone(), new_chain.id.clone());
            }
            chain_repo::ChainRepository::insert(conn, &new_chain)?;
        }

        // 2) 创建变量组
        let mut count = 0i32;
        for ge in &payload.groups {
            let now = chrono::Utc::now().timestamp();
            let resolved_chain = ge
                .chain_id
                .as_ref()
                .and_then(|id| chain_id_map.get(id).cloned());
            let group = EnvGroup {
                id: uuid::Uuid::new_v4().to_string(),
                name: ge.name.clone(),
                description: ge.description.clone(),
                variables: ge.variables.clone(),
                is_active: false,
                chain_id: resolved_chain,
                created_at: ge.created_at.unwrap_or(now),
                updated_at: now,
            };
            env_group_repo::EnvGroupRepository::insert(conn, &group)?;
            history_repo::HistoryRepository::insert(
                conn,
                &HistoryRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    action_type: "import".to_string(),
                    target_type: "group".to_string(),
                    target_id: group.id.clone(),
                    before_data: None,
                    after_data: Some(serde_json::to_string(&group)?),
                    timestamp: now,
                },
            )?;
            count += 1;
        }
        Ok(count)
    })?;

    Ok(count)
}

/// 批量删除变量组，如果已激活则先从系统中移除。
#[tauri::command]
pub async fn batch_delete_groups(
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> AppResult<i32> {
    if ids.is_empty() {
        return Ok(0);
    }

    // 先获取这些组的 is_active 状态
    let groups = state.with_db(|conn| -> AppResult<Vec<EnvGroup>> {
        let mut result = Vec::new();
        for id in &ids {
            if let Some(g) = env_group_repo::EnvGroupRepository::get_by_id(conn, id)? {
                result.push(g);
            }
        }
        Ok(result)
    })?;

    // 如果有激活的组，先逐个 deactivate
    for g in &groups {
        if g.is_active {
            let _ = EnvService::deactivate_group(&state, &g.id).await;
        }
    }

    let count = state.with_db(|conn| -> AppResult<i32> {
        let mut deleted = 0i32;
        let now = chrono::Utc::now().timestamp();
        for g in &groups {
            env_group_repo::EnvGroupRepository::delete(conn, &g.id)?;
            history_repo::HistoryRepository::insert(
                conn,
                &HistoryRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    action_type: "delete".to_string(),
                    target_type: "group".to_string(),
                    target_id: g.id.clone(),
                    before_data: Some(serde_json::to_string(g)?),
                    after_data: None,
                    timestamp: now,
                },
            )?;
            deleted += 1;
        }
        Ok(deleted)
    })?;

    Ok(count)
}

/// 激活变量组时，检测是否与当前系统变量冲突。
#[tauri::command]
pub async fn detect_conflicts(
    state: State<'_, AppState>,
    group_id: String,
) -> AppResult<Vec<crate::models::EnvVarConflict>> {
    let platform = &*state.platform;
    let group = state.with_db(|conn| -> AppResult<EnvGroup> {
        env_group_repo::EnvGroupRepository::get_by_id(conn, &group_id)?
            .ok_or_else(|| AppError::NotFound(format!("变量组 {} 不存在", group_id)))
    })?;
    let current_vars = platform.get_all_variables(false).await?;
    let mut conflicts = Vec::new();
    for var in &group.variables {
        if let Some(existing) = current_vars.iter().find(|v| v.name == var.name) {
            if existing.value != var.value {
                conflicts.push(crate::models::EnvVarConflict {
                    name: var.name.clone(),
                    existing_value: existing.value.clone(),
                    new_value: var.value.clone(),
                });
            }
        }
    }
    Ok(conflicts)
}
