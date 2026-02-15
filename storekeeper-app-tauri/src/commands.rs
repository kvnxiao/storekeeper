//! Tauri commands for frontend-backend communication.

use std::collections::HashMap;

use chrono::Utc;
use storekeeper_core::{AppConfig, GameId, SecretsConfig};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_notification::NotificationExt;

use crate::notification;
use crate::polling;
use crate::state::{AllDailyRewardStatus, AllResources, AppState};

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

/// Saves the application configuration.
#[tauri::command]
pub async fn save_config(config: AppConfig) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())?;
    tracing::info!("Configuration saved successfully");
    Ok(())
}

/// Gets the current secrets configuration.
#[tauri::command]
pub async fn get_secrets() -> Result<SecretsConfig, String> {
    SecretsConfig::load().map_err(|e| e.to_string())
}

/// Saves the secrets configuration.
#[tauri::command]
pub async fn save_secrets(secrets: SecretsConfig) -> Result<(), String> {
    secrets.save().map_err(|e| e.to_string())?;
    tracing::info!("Secrets saved successfully");
    Ok(())
}

/// Reloads configuration and restarts the polling loop with new settings.
///
/// This should be called after saving config/secrets to apply the changes.
#[tauri::command]
pub async fn reload_config(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<AppState>();

    // Reload config and reinitialize game clients
    state.reload_config().await;

    // Trigger an immediate refresh to fetch resources with new config
    let _ = polling::refresh_now(&app_handle).await;

    Ok(())
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

// ============================================================================
// Daily Reward Commands
// ============================================================================

/// Gets the cached daily reward status for all games.
#[tauri::command]
pub async fn get_daily_reward_status(
    state: State<'_, AppState>,
) -> Result<AllDailyRewardStatus, String> {
    Ok(state.get_daily_reward_status().await)
}

/// Refreshes the daily reward status from all configured games.
#[tauri::command]
pub async fn refresh_daily_reward_status(
    state: State<'_, AppState>,
) -> Result<AllDailyRewardStatus, String> {
    let status = state.fetch_all_daily_reward_status().await;
    state.set_daily_reward_status(status.clone()).await;
    Ok(status)
}

/// Claims daily rewards for all configured games.
///
/// Returns a map of game ID to claim result.
#[tauri::command]
pub async fn claim_daily_rewards(
    state: State<'_, AppState>,
) -> Result<HashMap<GameId, serde_json::Value>, String> {
    tracing::info!("Manual daily reward claim requested");
    let results = state.claim_all_daily_rewards().await;

    // Refresh status after claiming
    let status = state.fetch_all_daily_reward_status().await;
    state.set_daily_reward_status(status).await;

    Ok(results)
}

/// Claims daily reward for a specific game.
#[tauri::command]
pub async fn claim_daily_reward_for_game(
    game_id: GameId,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    tracing::info!(game_id = ?game_id, "Manual daily reward claim requested for specific game");
    let result = state.claim_daily_reward_for_game(game_id).await?;

    // Refresh status for this game after claiming
    if let Ok(game_status) = state.get_daily_reward_status_for_game(game_id).await {
        let mut current_status = state.get_daily_reward_status().await;
        current_status.games.insert(game_id, game_status);
        current_status.last_checked = Some(chrono::Utc::now());
        state.set_daily_reward_status(current_status).await;
    }

    Ok(result)
}

/// Gets the daily reward status for a specific game.
#[tauri::command]
pub async fn get_daily_reward_status_for_game(
    game_id: GameId,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    state.get_daily_reward_status_for_game(game_id).await
}

// ============================================================================
// Notification Commands
// ============================================================================

/// Sends a test notification for a specific game resource using cached data.
#[tauri::command]
pub async fn send_test_notification(
    game_id: GameId,
    resource_type: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let resources = state.get_resources().await;
    let game_name = game_id.display_name();
    let resource_name = notification::resource_display_name(game_id, &resource_type);

    // Try to find cached resource data and build a real notification body
    let body = resources
        .games
        .get(&game_id)
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter().find(|obj| {
                obj.get("type")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|t| t == resource_type)
            })
        })
        .and_then(|obj| obj.get("data"))
        .and_then(|data| {
            let info = notification::extract_resource_info(data)?;
            let now = Utc::now();
            let time_to_full = info.completion_at - now;
            Some(if info.is_complete {
                let overdue_mins = (now - info.completion_at).num_minutes();
                if overdue_mins <= 0 {
                    format!("{resource_name} is full!")
                } else {
                    format!("{resource_name} has been full for {overdue_mins} minutes")
                }
            } else {
                let mins_remaining = time_to_full.num_minutes();
                format!("{resource_name} will be full in {mins_remaining} minutes")
            })
        })
        .unwrap_or_else(|| {
            format!("{resource_name} — No data available yet, try refreshing first")
        });

    app_handle
        .notification()
        .builder()
        .title(format!("{game_name} — {resource_name}"))
        .body(&body)
        .show()
        .map_err(|e| e.to_string())
}
