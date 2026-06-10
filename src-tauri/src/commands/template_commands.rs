use crate::error::AppResult;
use crate::models::Template;
use crate::repositories::template_repo;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_all_templates(state: State<'_, AppState>) -> AppResult<Vec<Template>> {
    state.with_db(|conn| template_repo::TemplateRepository::get_all(conn).map_err(Into::into))
}

#[tauri::command]
pub async fn create_template(
    state: State<'_, AppState>,
    name: String,
    keys: Vec<String>,
) -> AppResult<Template> {
    if name.trim().is_empty() {
        return Err(crate::error::AppError::Validation(
            "模板名称不能为空".to_string(),
        ));
    }
    let now = chrono::Utc::now().timestamp();
    let cleaned_keys: Vec<String> = keys
        .into_iter()
        .filter(|k| !k.trim().is_empty())
        .map(|k| k.trim().to_string())
        .collect();
    if cleaned_keys.is_empty() {
        return Err(crate::error::AppError::Validation(
            "请至少添加一个变量名".to_string(),
        ));
    }
    let tpl = Template {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        keys: cleaned_keys,
        created_at: now,
        updated_at: now,
    };

    state.with_db(|conn| {
        template_repo::TemplateRepository::insert(conn, &tpl)?;
        Ok::<_, crate::error::AppError>(())
    })?;

    Ok(tpl)
}

#[tauri::command]
pub async fn update_template(
    state: State<'_, AppState>,
    id: String,
    name: String,
    keys: Vec<String>,
) -> AppResult<Template> {
    if name.trim().is_empty() {
        return Err(crate::error::AppError::Validation(
            "模板名称不能为空".to_string(),
        ));
    }
    let now = chrono::Utc::now().timestamp();
    let cleaned_keys: Vec<String> = keys
        .into_iter()
        .filter(|k| !k.trim().is_empty())
        .map(|k| k.trim().to_string())
        .collect();
    if cleaned_keys.is_empty() {
        return Err(crate::error::AppError::Validation(
            "请至少添加一个变量名".to_string(),
        ));
    }

    let updated = state.with_db(|conn| -> AppResult<Template> {
        let existing = template_repo::TemplateRepository::get_by_id(conn, &id)?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("模板 {} 不存在", id)))?;
        let tpl = Template {
            id: existing.id.clone(),
            name: name.trim().to_string(),
            keys: cleaned_keys,
            created_at: existing.created_at,
            updated_at: now,
        };
        template_repo::TemplateRepository::update(conn, &tpl)?;
        Ok(tpl)
    })?;

    Ok(updated)
}

#[tauri::command]
pub async fn delete_template(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.with_db(|conn| {
        template_repo::TemplateRepository::delete(conn, &id)?;
        Ok::<_, crate::error::AppError>(())
    })?;
    Ok(())
}
