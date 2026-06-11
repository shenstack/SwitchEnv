mod commands;
mod db;
mod error;
mod models;
mod platforms;
mod repositories;
mod services;
mod state;

use state::AppState;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("SwitchEnv.db");
            let mut conn = rusqlite::Connection::open(&db_path).expect("failed to open database");
            db::run_migrations(&mut conn).expect("failed to run migrations");

            let platform: Arc<dyn PlatformService> = Arc::from(platforms::create_platform_service());
            let state = AppState::new(conn, platform);
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::env_commands::get_all_env_vars,
            commands::env_commands::get_env_var,
            commands::env_commands::set_env_var,
            commands::env_commands::remove_env_var,
            commands::env_commands::can_modify_system,
            commands::env_commands::refresh_environment,
            commands::env_commands::open_system_settings,
            commands::env_commands::get_shell_config_info,
            commands::group_commands::get_all_groups,
            commands::group_commands::create_group,
            commands::group_commands::update_group,
            commands::group_commands::delete_group,
            commands::group_commands::activate_group,
            commands::group_commands::deactivate_group,
            commands::template_commands::get_all_templates,
            commands::template_commands::create_template,
            commands::template_commands::update_template,
            commands::template_commands::delete_template,
            commands::group_io_commands::export_groups,
            commands::group_io_commands::import_groups,
            commands::group_io_commands::batch_delete_groups,
            commands::group_io_commands::detect_conflicts,
            commands::history_commands::get_history,
            commands::history_commands::restore_history,
            commands::history_commands::clear_history,
            commands::backup_commands::create_backup,
            commands::backup_commands::get_all_backups,
            commands::backup_commands::restore_backup,
            commands::backup_commands::delete_backup,
            commands::backup_commands::export_backup,
            commands::backup_commands::import_backup,
            commands::settings_commands::get_app_settings,
            commands::settings_commands::set_app_settings,
            commands::utils_commands::copy_to_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use crate::platforms::PlatformService;
