use crate::error::AppResult;
use crate::models::*;
use crate::repositories::{env_group_repo, history_repo};
use crate::state::AppState;

pub struct EnvService;

impl EnvService {
    /// 激活一个环境变量组：
    /// - 先检测冲突（与系统用户环境注册表 & 其它已激活组的同名变量）；
    /// - 若 force=false 且存在冲突，则返回 success=false 并返回所有冲突项，不写入系统；
    /// - 若 force=true 或无冲突，则写入系统变量 & 刷新环境 & 更新 DB 激活状态；
    /// - 不再处理 chain，互斥逻辑已移除。
    pub async fn activate_group(
        state: &AppState,
        group_id: &str,
        force: bool,
    ) -> AppResult<ActivationResult> {
        let total_start = std::time::Instant::now();
        log::info!("[activate_group] [group_id={}] force={} 开始执行", group_id, force);

        // 1. 读取目标组与其它已激活组
        let step = std::time::Instant::now();
        let (group, other_active) = state.with_db(|conn| -> AppResult<(EnvGroup, Vec<EnvGroup>)> {
            let g = env_group_repo::EnvGroupRepository::get_by_id(conn, group_id)?
                .ok_or_else(|| crate::error::AppError::NotFound(format!("变量组 {} 不存在", group_id)))?;
            let all = env_group_repo::EnvGroupRepository::get_all(conn)?;
            let others = all.into_iter().filter(|x| x.is_active && x.id != group_id).collect();
            Ok((g, others))
        })?;
        log::info!("[activate_group] [group_id={}] 步骤1: 读取目标组+其他激活组: {:.2}ms",
            group_id, step.elapsed().as_secs_f64() * 1000.0);

        let platform = &*state.platform;

        // 2. 读取系统用户环境变量（用于冲突检测 A）
        let step = std::time::Instant::now();
        let current_sys = platform.get_all_variables(false).await?;
        log::info!("[activate_group] [group_id={}] 步骤2: 读取系统用户环境变量({}项): {:.2}ms",
            group_id, current_sys.len(), step.elapsed().as_secs_f64() * 1000.0);

        let mut conflicts: Vec<EnvVarConflict> = Vec::new();

        // 3. 冲突检测（与系统变量 + 与其他激活组）
        let step = std::time::Instant::now();
        for var in &group.variables {
            // A. 系统环境（用户环境变量注册表）
            if let Some(existing) = current_sys.iter().find(|v| v.name == var.name) {
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
            // B. 与其它已激活组
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
        log::info!("[activate_group] [group_id={}] 步骤3: 冲突检测({}个变量, {}个冲突): {:.2}ms",
            group_id, group.variables.len(), conflicts.len(), step.elapsed().as_secs_f64() * 1000.0);

        if !force && !conflicts.is_empty() {
            // 有冲突且未强制写入 -> 返回 conflicts 给前端，不做写入
            log::info!("[activate_group] [group_id={}] 检测到冲突, early return, 总耗时: {:.2}ms",
                group_id, total_start.elapsed().as_secs_f64() * 1000.0);
            return Ok(ActivationResult {
                success: false,
                conflicts,
                errors: Vec::new(),
            });
        }

        // 4. 真正写入系统变量
        let step = std::time::Instant::now();
        let mut errors: Vec<String> = Vec::new();
        for var in &group.variables {
            if let Err(e) = platform.set_variable(&var.name, &var.value, false).await {
                errors.push(format!("写入变量 {} 失败: {}", var.name, e));
            }
        }
        log::info!("[activate_group] [group_id={}] 步骤4: 写入系统变量({}个变量, {}个错误): {:.2}ms",
            group_id, group.variables.len(), errors.len(), step.elapsed().as_secs_f64() * 1000.0);

        // 5. 刷新系统环境（广播 WM_SETTINGCHANGE）
        let step = std::time::Instant::now();
        platform.refresh_environment().await?;
        log::info!("[activate_group] [group_id={}] 步骤5: 刷新环境(广播 WM_SETTINGCHANGE): {:.2}ms",
            group_id, step.elapsed().as_secs_f64() * 1000.0);

        // 6. 更新 DB 状态 + 写历史记录
        let step = std::time::Instant::now();
        let before_json = serde_json::to_string(&group)?;
        state.with_db(|conn| -> AppResult<()> {
            let mut updated = group.clone();
            updated.is_active = true;
            updated.updated_at = chrono::Utc::now().timestamp();
            env_group_repo::EnvGroupRepository::update(conn, &updated)?;
            history_repo::HistoryRepository::insert(
                conn,
                &HistoryRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    action_type: "activate".to_string(),
                    target_type: "group".to_string(),
                    target_id: group_id.to_string(),
                    before_data: Some(before_json),
                    after_data: Some(serde_json::to_string(&updated)?),
                    timestamp: chrono::Utc::now().timestamp(),
                },
            )?;
            Ok(())
        })?;
        log::info!("[activate_group] [group_id={}] 步骤6: 更新DB状态+写历史: {:.2}ms",
            group_id, step.elapsed().as_secs_f64() * 1000.0);

        log::info!("[activate_group] [group_id={}] 激活完成, 总耗时: {:.2}ms",
            group_id, total_start.elapsed().as_secs_f64() * 1000.0);
        Ok(ActivationResult {
            success: errors.is_empty(),
            conflicts,
            errors,
        })
    }

    /// 停用一个环境变量组：从系统中移除，更新 DB 状态并写入历史记录。
    pub async fn deactivate_group(state: &AppState, group_id: &str) -> AppResult<()> {
        let total_start = std::time::Instant::now();
        log::info!("[deactivate_group] [group_id={}] 开始执行", group_id);

        // 步骤1: 从 DB 读取目标组
        let step = std::time::Instant::now();
        let (group, before_json) = state.with_db(|conn| -> AppResult<(EnvGroup, String)> {
            let group = env_group_repo::EnvGroupRepository::get_by_id(conn, group_id)?
                .ok_or_else(|| crate::error::AppError::NotFound(format!("变量组 {} 不存在", group_id)))?;
            let before = serde_json::to_string(&group)?;
            Ok((group, before))
        })?;
        log::info!("[deactivate_group] [group_id={}] 步骤1: 读取目标组: {:.2}ms",
            group_id, step.elapsed().as_secs_f64() * 1000.0);

        if !group.is_active {
            log::info!("[deactivate_group] [group_id={}] 已处于未激活状态, early return, 总耗时: {:.2}ms",
                group_id, total_start.elapsed().as_secs_f64() * 1000.0);
            return Ok(());
        }

        // 步骤2: 从注册表删除变量
        let step = std::time::Instant::now();
        let platform = &*state.platform;
        for var in &group.variables {
            let _ = platform.remove_variable(&var.name, false).await;
        }
        log::info!("[deactivate_group] [group_id={}] 步骤2: 从注册表删除变量({}个): {:.2}ms",
            group_id, group.variables.len(), step.elapsed().as_secs_f64() * 1000.0);

        // 步骤3: 刷新环境（广播 WM_SETTINGCHANGE）
        let step = std::time::Instant::now();
        platform.refresh_environment().await?;
        log::info!("[deactivate_group] [group_id={}] 步骤3: 刷新环境(广播 WM_SETTINGCHANGE): {:.2}ms",
            group_id, step.elapsed().as_secs_f64() * 1000.0);

        // 步骤4: 更新 DB 状态 + 写历史记录
        let step = std::time::Instant::now();
        state.with_db(|conn| -> AppResult<()> {
            let mut updated = group.clone();
            updated.is_active = false;
            updated.updated_at = chrono::Utc::now().timestamp();
            env_group_repo::EnvGroupRepository::update(conn, &updated)?;
            history_repo::HistoryRepository::insert(
                conn,
                &HistoryRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    action_type: "deactivate".to_string(),
                    target_type: "group".to_string(),
                    target_id: group_id.to_string(),
                    before_data: Some(before_json),
                    after_data: None,
                    timestamp: chrono::Utc::now().timestamp(),
                },
            )?;
            Ok(())
        })?;
        log::info!("[deactivate_group] [group_id={}] 步骤4: 更新DB状态+写历史: {:.2}ms",
            group_id, step.elapsed().as_secs_f64() * 1000.0);

        log::info!("[deactivate_group] [group_id={}] 停用完成, 总耗时: {:.2}ms",
            group_id, total_start.elapsed().as_secs_f64() * 1000.0);
        Ok(())
    }
}
