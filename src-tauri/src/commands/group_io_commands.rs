use crate::error::{AppError, AppResult};
use crate::models::{
    EnvGroup, EnvVarConflict, GroupExport, GroupExportPayload, HistoryRecord, ImportConflictGroup,
    ImportPreviewResult, ImportVarDiff,
};
use crate::repositories::{env_group_repo, history_repo};
use crate::services::env_service::EnvService;
use crate::state::AppState;
use std::collections::HashMap;
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

/// 解析导入的 JSON 并按组名与现有变量组进行比对，返回新增和冲突组的清单。
/// 冲突组内包含逐变量差异，用于前端弹窗展示。
#[tauri::command]
pub async fn preview_import_groups(
    state: State<'_, AppState>,
    json: String,
) -> AppResult<ImportPreviewResult> {
    let payload: GroupExportPayload = serde_json::from_str(&json).map_err(|e| {
        AppError::Validation(format!("导入的 JSON 格式不正确: {}", e))
    })?;
    if payload.type_ != "env-groups-export" || payload.groups.is_empty() {
        return Err(AppError::Validation(
            "文件格式不正确或未包含变量组".to_string(),
        ));
    }

    let result = state.with_db(|conn| -> AppResult<ImportPreviewResult> {
        let existing = env_group_repo::EnvGroupRepository::get_all(conn)?;
        let existing_map: HashMap<String, &EnvGroup> = existing
            .iter()
            .map(|g| (g.name.clone(), g))
            .collect();

        let mut new_groups = Vec::new();
        let mut conflict_groups = Vec::new();

        for ge in &payload.groups {
            if let Some(existing_group) = existing_map.get(&ge.name) {
                // 名称已存在，进行变量差异比对
                let existing_var_map: HashMap<String, &crate::models::EnvVariable> =
                    existing_group.variables.iter().map(|v| (v.name.clone(), v)).collect();
                let incoming_var_map: HashMap<String, &crate::models::EnvVariable> =
                    ge.variables.iter().map(|v| (v.name.clone(), v)).collect();

                let mut all_names: Vec<String> = existing_var_map
                    .keys()
                    .chain(incoming_var_map.keys())
                    .cloned()
                    .collect();
                all_names.sort();
                all_names.dedup();

                let mut var_diffs = Vec::new();
                let mut all_identical = true;

                for name in all_names {
                    let existing_var = existing_var_map.get(&name);
                    let incoming_var = incoming_var_map.get(&name);

                    match (existing_var, incoming_var) {
                        (Some(ev), Some(iv)) => {
                            if ev.value != iv.value || ev.is_hidden != iv.is_hidden {
                                var_diffs.push(ImportVarDiff {
                                    name: name.clone(),
                                    diff_type: "value_changed".to_string(),
                                    existing_value: Some(ev.value.clone()),
                                    incoming_value: Some(iv.value.clone()),
                                    existing_is_hidden: Some(ev.is_hidden),
                                    incoming_is_hidden: Some(iv.is_hidden),
                                });
                                all_identical = false;
                            }
                        }
                        (Some(ev), None) => {
                            var_diffs.push(ImportVarDiff {
                                name: name.clone(),
                                diff_type: "missing_only_existing".to_string(),
                                existing_value: Some(ev.value.clone()),
                                incoming_value: None,
                                existing_is_hidden: Some(ev.is_hidden),
                                incoming_is_hidden: None,
                            });
                            all_identical = false;
                        }
                        (None, Some(iv)) => {
                            var_diffs.push(ImportVarDiff {
                                name: name.clone(),
                                diff_type: "added_only_incoming".to_string(),
                                existing_value: None,
                                incoming_value: Some(iv.value.clone()),
                                existing_is_hidden: None,
                                incoming_is_hidden: Some(iv.is_hidden),
                            });
                            all_identical = false;
                        }
                        (None, None) => {}
                    }
                }

                conflict_groups.push(ImportConflictGroup {
                    name: ge.name.clone(),
                    existing_description: existing_group.description.clone(),
                    incoming_description: ge.description.clone(),
                    var_diffs,
                    is_identical: all_identical,
                });
            } else {
                new_groups.push(GroupExport {
                    name: ge.name.clone(),
                    description: ge.description.clone(),
                    variables: ge.variables.clone(),
                    created_at: ge.created_at,
                });
            }
        }

        Ok(ImportPreviewResult {
            new_groups,
            conflict_groups,
        })
    })?;

    Ok(result)
}

/// 根据用户决策执行实际的导入操作。
/// overwrite_names：要覆盖（更新）的组名；
/// merge_names：要合并的组名（变量名取并集，同名变量以导入值为准，描述使用导入描述）；
/// ignore_names：要忽略跳过的组名；
/// 未在三个列表中且数据库不存在的组名会被直接作为新组 INSERT。
#[tauri::command]
pub async fn execute_import_groups(
    state: State<'_, AppState>,
    json: String,
    overwrite_names: Vec<String>,
    merge_names: Vec<String>,
    ignore_names: Vec<String>,
) -> AppResult<i32> {
    let payload: GroupExportPayload = serde_json::from_str(&json).map_err(|e| {
        AppError::Validation(format!("导入的 JSON 格式不正确: {}", e))
    })?;
    if payload.type_ != "env-groups-export" || payload.groups.is_empty() {
        return Err(AppError::Validation(
            "文件格式不正确或未包含变量组".to_string(),
        ));
    }

    let count = state.with_db(|conn| -> AppResult<i32> {
        let existing = env_group_repo::EnvGroupRepository::get_all(conn)?;
        let existing_map: HashMap<String, &EnvGroup> =
            existing.iter().map(|g| (g.name.clone(), g)).collect();

        let overwrite_set: std::collections::HashSet<String> =
            overwrite_names.into_iter().collect();
        let merge_set: std::collections::HashSet<String> =
            merge_names.into_iter().collect();
        let ignore_set: std::collections::HashSet<String> =
            ignore_names.into_iter().collect();

        let mut count = 0i32;
        let now = chrono::Utc::now().timestamp();

        for ge in &payload.groups {
            if ignore_set.contains(&ge.name) {
                continue;
            }

            if overwrite_set.contains(&ge.name) {
                // 覆盖：更新现有记录
                if let Some(existing_group) = existing_map.get(&ge.name) {
                    let before = Some(serde_json::to_string(&existing_group)?);
                    let updated = EnvGroup {
                        id: existing_group.id.clone(),
                        name: ge.name.clone(),
                        description: ge.description.clone(),
                        variables: ge.variables.clone(),
                        is_active: existing_group.is_active,
                        created_at: existing_group.created_at,
                        updated_at: now,
                    };
                    env_group_repo::EnvGroupRepository::update(conn, &updated)?;
                    history_repo::HistoryRepository::insert(
                        conn,
                        &HistoryRecord {
                            id: uuid::Uuid::new_v4().to_string(),
                            action_type: "import".to_string(),
                            target_type: "group".to_string(),
                            target_id: updated.id.clone(),
                            before_data: before,
                            after_data: Some(serde_json::to_string(&updated)?),
                            timestamp: now,
                        },
                    )?;
                    count += 1;
                }
            } else if merge_set.contains(&ge.name) {
                // 合并：变量以名为键取并集，同名变量以导入值为准；描述使用导入描述
                if let Some(existing_group) = existing_map.get(&ge.name) {
                    use crate::models::EnvVariable;
                    let before = Some(serde_json::to_string(&existing_group)?);
                    let mut merged_vars: Vec<EnvVariable> = Vec::new();
                    let mut seen: std::collections::HashSet<String> =
                        std::collections::HashSet::new();
                    let incoming_map: std::collections::HashMap<String, &EnvVariable> =
                        ge.variables.iter().map(|v| (v.name.clone(), v)).collect();

                    // 先按现有顺序遍历：同名 → 取导入值；不同名 → 保留现有
                    for ev in &existing_group.variables {
                        if seen.contains(&ev.name) {
                            continue;
                        }
                        seen.insert(ev.name.clone());
                        if let Some(iv) = incoming_map.get(&ev.name) {
                            merged_vars.push(EnvVariable {
                                name: iv.name.clone(),
                                value: iv.value.clone(),
                                is_hidden: iv.is_hidden,
                            });
                        } else {
                            merged_vars.push(ev.clone());
                        }
                    }

                    // 再追加导入中新增（仅导入有的）
                    for iv in &ge.variables {
                        if !seen.contains(&iv.name) {
                            seen.insert(iv.name.clone());
                            merged_vars.push(iv.clone());
                        }
                    }

                    let updated = EnvGroup {
                        id: existing_group.id.clone(),
                        name: ge.name.clone(),
                        description: ge.description.clone(),
                        variables: merged_vars,
                        is_active: existing_group.is_active,
                        created_at: existing_group.created_at,
                        updated_at: now,
                    };
                    env_group_repo::EnvGroupRepository::update(conn, &updated)?;
                    history_repo::HistoryRepository::insert(
                        conn,
                        &HistoryRecord {
                            id: uuid::Uuid::new_v4().to_string(),
                            action_type: "import".to_string(),
                            target_type: "group".to_string(),
                            target_id: updated.id.clone(),
                            before_data: before,
                            after_data: Some(serde_json::to_string(&updated)?),
                            timestamp: now,
                        },
                    )?;
                    count += 1;
                }
            } else if !existing_map.contains_key(&ge.name) {
                // 名称不在三个列表中且数据库中不存在 → INSERT 新增
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
            // 否则（名称不在两个列表中但数据库存在 → 保守跳过，不意外覆盖
        }

        Ok(count)
    })?;

    Ok(count)
}
