use crate::error::{AppError, AppResult};
use crate::models::{EnvGroup, EnvVarConflict, GroupExport, GroupExportPayload, HistoryRecord};
use crate::repositories::{env_group_repo, history_repo};
use crate::services::env_service::EnvService;
use crate::state::AppState;
use std::path::PathBuf;
use tauri::State;

/// 导出指定变量组。
#[tauri::command]
pub async fn export_groups(
    state: State<'_, AppState>,
    group_ids: Option<Vec<String>>,
    save_path: Option<PathBuf>,
) -> AppResult<String> {
    let groups = state.with_db(|conn| -> AppResult<Vec<EnvGroup>> {
        let all = env_group_repo::EnvGroupRepository::get_all(conn)?;
        let filtered = if let Some(ids) = &group_ids {
            all.into_iter().filter(|g| ids.contains(&g.id)).collect()
        } else {
            all
        };
        Ok(filtered)
    })?;

    let now = chrono::Utc::now().timestamp();
    let payload = GroupExportPayload {
        type_: "env-groups-export".to_string(),
        version: 3,
        exported_at: now,
        groups: groups
            .iter()
            .map(|g| GroupExport {
                name: g.name.clone(),
                description: g.description.clone(),
                variables: g.variables.clone(),
                created_at: Some(g.created_at),
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&payload).map_err(|e| AppError::Other(e.to_string()))?;

    if let Some(path) = save_path {
        std::fs::write(&path, &json).map_err(AppError::Io)?;
        let abs = std::fs::canonicalize(&path).unwrap_or(path);
        return Ok(abs.to_string_lossy().to_string());
    }
    Ok(json)
}

/// 从 JSON 字符串导入变量组。
/// 向后兼容：旧文件中的未知字段（chains / groups[i].chainId）会被 serde 自动忽略。
#[tauri::command]
pub async fn import_groups(state: State<'_, AppState>, json: String) -> AppResult<i32> {
    let payload: GroupExportPayload = serde_json::from_str(&json).map_err(|e| {
        AppError::Validation(format!("导入的 JSON 格式不正确: {}", e))
    })?;
    if payload.type_ != "env-groups-export" || payload.groups.is_empty() {
        return Err(AppError::Validation(
            "文件格式不正确或未包含变量组".to_string(),
        ));
    }

    let count = state.with_db(|conn| -> AppResult<i32> {
        let mut count = 0i32;
        let now = chrono::Utc::now().timestamp();
        for ge in &payload.groups {
            let group = EnvGroup {
                id: uuid::Uuid::new_v4().to_string(),
                name: ge.name.clone(),
                description: ge.description.clone(),
                variables: ge.variables.clone(),
                is_active: false,
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

    let groups = state.with_db(|conn| -> AppResult<Vec<EnvGroup>> {
        let mut result = Vec::new();
        for id in &ids {
            if let Some(g) = env_group_repo::EnvGroupRepository::get_by_id(conn, id)? {
                result.push(g);
            }
        }
        Ok(result)
    })?;

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

/// 检测变量组与系统环境（用户环境变量）的冲突。
/// 注意：前端激活流程中会直接调用 activate_group(group_id, force=false)，
/// 该命令会在后端同时完成「系统检测+其他已激活组检测」并返回 conflicts，
/// 因此本命令主要用于 UI 上预先查看冲突（如在详情页展示当前与系统变量的差异）。
#[tauri::command]
pub async fn detect_conflicts(
    state: State<'_, AppState>,
    group_id: String,
) -> AppResult<Vec<EnvVarConflict>> {
    let platform = &*state.platform;
    let (group, other_active) = state.with_db(|conn| -> AppResult<(EnvGroup, Vec<EnvGroup>)> {
        let g = env_group_repo::EnvGroupRepository::get_by_id(conn, &group_id)?
            .ok_or_else(|| AppError::NotFound(format!("变量组 {} 不存在", group_id)))?;
        let all = env_group_repo::EnvGroupRepository::get_all(conn)?;
        let others = all.into_iter().filter(|x| x.is_active && x.id != group_id).collect();
        Ok((g, others))
    })?;

    let current_vars = platform.get_all_variables(false).await?;
    let mut conflicts = Vec::new();

    for var in &group.variables {
        // 与系统用户环境变量（注册表）比对
        if let Some(existing) = current_vars.iter().find(|v| v.name == var.name) {
            if existing.value != var.value {
                conflicts.push(EnvVarConflict {
                    name: var.name.clone(),
                    existing_value: existing.value.clone(),
                    new_value: var.value.clone(),
                    source: "system".to_string(),
                    source_group_name: None,
                });
            }
        }
        // 与其它已激活组比对
        for other in &other_active {
            if let Some(existing_in_group) = other.variables.iter().find(|v| v.name == var.name) {
                if existing_in_group.value != var.value {
                    conflicts.push(EnvVarConflict {
                        name: var.name.clone(),
                        existing_value: existing_in_group.value.clone(),
                        new_value: var.value.clone(),
                        source: other.id.clone(),
                        source_group_name: Some(other.name.clone()),
                    });
                }
            }
        }
    }

    Ok(conflicts)
}
