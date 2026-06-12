use crate::error::{AppError, AppResult};
use std::process::Command;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn copy_to_clipboard(app: AppHandle, text: String) -> AppResult<()> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| AppError::Clipboard(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn open_path(path: String) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(&path).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer.exe").arg(&path).spawn()?;
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        Command::new("xdg-open").arg(&path).spawn()?;
    }
    Ok(())
}
