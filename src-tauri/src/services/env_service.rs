use crate::error::AppResult;
use crate::models::*;
use crate::repositories::{env_group_repo, history_repo};
use crate::state::AppState;

pub struct EnvService;

impl EnvService {
    /// 激活一个环境变量组：写入系统 / 用户 / Shell 配置中，
    /// 处理链条互斥（同 chain 内同时只激活一个），并写入历史记录。
    pub async fn activate_group(state: &AppState, group_id: &str) -> AppResult<ActivationResult> {
        // 第一步：从 DB 读取所有必要数据（同步闭包）
        let (mut group, chain_active, maybe_before_json) = state.with_db(|conn| {
            let group = env_group_repo::EnvGroupRepository::get_by_id(conn, group_id)?
                .ok_or_else(|| crate::error::AppError::NotFound(format!("变量组 {} 不存在", group_id)))?;
            let chain_active = if let Some(ref chain) = group.chain_id {
                if !chain.is_empty() {
                    env_group_repo::EnvGroupRepository::get_active_by_chain(conn, chain)?
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };
            let before = serde_json::to_string(&group)?;
            Ok((group, chain_active, before))
        })?;

        let platform = &*state.platform;
        let mut errors: Vec<String> = Vec::new();

        // 1) 处理链条：先停用链条内其它已激活的组
        let mut deactivated_ids = Vec::new();
        for other in &chain_active {
            if other.id == group_id {
                continue;
            }
            for var in &other.variables {
                if let Err(e) = platform.remove_variable(&var.name, false).await {
                    errors.push(format!("停用变量 {} 失败: {}", var.name, e));
                }
            }
            // 更新状态 & 历史记录（同步）
            let other_id = other.id.clone();
            state.with_db(move |conn| {
                let mut updated = other.clone();
                updated.is_active = false;
                updated.updated_at = chrono::Utc::now().timestamp();
                env_group_repo::EnvGroupRepository::update(conn, &updated)?;
                history_repo::HistoryRepository::insert(
                    conn,
                    &HistoryRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        action_type: "deactivate".to_string(),
                        target_type: "group".to_string(),
                        target_id: other_id,
                        before_data: Some(serde_json::to_string(&updated)?),
                        after_data: None,
                        timestamp: chrono::Utc::now().timestamp(),
                    },
                )?;
                Ok(())
            })?;
            deactivated_ids.push(other.id.clone());
        }

        // 2) 检测同名冲突
        let current_vars = platform.get_all_variables(false).await?;
        let mut conflicts: Vec<EnvVarConflict> = Vec::new();
        for var in &group.variables {
            if let Some(existing) = current_vars.iter().find(|v| v.name == var.name) {
                if existing.value != var.value {
                    conflicts.push(EnvVarConflict {
                        name: var.name.clone(),
                        existing_value: existing.value.clone(),
                        new_value: var.value.clone(),
                    });
                }
            }
        }

        // 3) 应用本组变量
        for var in &group.variables {
            if let Err(e) = platform.set_variable(&var.name, &var.value, false).await {
                errors.push(format!("设置变量 {} 失败: {}", var.name, e));
            }
        }
        platform.refresh_environment().await?;

        // 4) 更新本组状态为已激活（同步）
        let after_json = state.with_db(|conn| {
            group.is_active = true;
            group.updated_at = chrono::Utc::now().timestamp();
            env_group_repo::EnvGroupRepository::update(conn, &group)?;
            history_repo::HistoryRepository::insert(
                conn,
                &HistoryRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    action_type: "activate".to_string(),
                    target_type: "group".to_string(),
                    target_id: group_id.to_string(),
                    before_data: Some(maybe_before_json),
                    after_data: Some(serde_json::to_string(&group)?),
                    timestamp: chrono::Utc::now().timestamp(),
                },
            )?;
            Ok::<_, crate::error::AppError>(serde_json::to_string(&group)?)
        })?;
        let _ = after_json; // 保留 future 扩展

        Ok(ActivationResult {
            success: errors.is_empty(),
            conflicts,
            deactivated_groups: deactivated_ids,
            errors,
        })
    }

    /// 停用一个环境变量组：从系统 / Shell 中移除，更新 DB 状态并写入历史。
    pub async fn deactivate_group(state: &AppState, group_id: &str) -> AppResult<()> {
        let (group, before_json) = state.with_db(|conn| {
            let group = env_group_repo::EnvGroupRepository::get_by_id(conn, group_id)?
                .ok_or_else(|| crate::error::AppError::NotFound(format!("变量组 {} 不存在", group_id)))?;
            let before = serde_json::to_string(&group)?;
            Ok::<_, crate::error::AppError>((group, before))
        })?;

        if !group.is_active {
            return Ok(());
        }

        let platform = &*state.platform;
        for var in &group.variables {
            let _ = platform.remove_variable(&var.name, false).await;
        }
        platform.refresh_environment().await?;

        state.with_db(|conn| {
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

        Ok(())
    }
}
