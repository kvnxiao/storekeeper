//! Tauri commands for frontend-backend communication.

use storekeeper_core::AppConfig;
use tauri::{AppHandle, State};

use crate::polling;
use crate::state::{AllResources, AppState};

/// Gets all cached resources.
#[tauri::command]
pub async fn get_all_resources(state: State<'_, AppState>) -> Result<AllResources, String> {
    Ok(state.get_resources().await)
}

/// Refreshes resources from all configured games.
#[tauri::command]
pub async fn refresh_resources(app_handle: AppHandle) -> Result<AllResources, String> {
    polling::refresh_now(&app_handle).await
}

/// Gets the current application configuration.
#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    AppConfig::load().map_err(|e| e.to_string())
}

/// Opens the configuration folder in the system file manager.
#[tauri::command]
pub fn open_config_folder() -> Result<(), String> {
    let config_dir = storekeeper_core::AppConfig::config_dir().map_err(|e| e.to_string())?;

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    }

    // Open in file manager
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
