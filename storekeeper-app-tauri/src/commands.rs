//! Tauri commands for frontend-backend communication.

use std::collections::HashMap;

use chrono::Utc;
use serde::Serialize;
use storekeeper_core::{AppConfig, GameId, SecretsConfig};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;

use crate::error::{CommandError, ErrorCode};
use crate::i18n;
use crate::notification;
use crate::polling;
use crate::state::{AllDailyRewardStatus, AllResources, AppState};

/// Gets all cached resources.
#[tauri::command]
pub async fn get_all_resources(state: State<'_, AppState>) -> Result<AllResources, CommandError> {
    Ok(state.get_resources().await)
}

/// Refreshes resources from all configured games.
#[tauri::command]
pub async fn refresh_resources(app_handle: AppHandle) -> Result<AllResources, CommandError> {
    polling::refresh_now(&app_handle)
        .await
        .map_err(CommandError::internal)
}

/// Gets the current application configuration.
#[tauri::command]
pub async fn get_config() -> Result<AppConfig, CommandError> {
    Ok(AppConfig::load()?)
}

/// Gets the current secrets configuration.
#[tauri::command]
pub async fn get_secrets() -> Result<SecretsConfig, CommandError> {
    Ok(SecretsConfig::load()?)
}

/// Result returned by `save_and_apply`.
#[derive(Serialize)]
pub struct SaveResult {
    /// The effective locale after applying changes.
    effective_locale: String,
}

/// Saves config + secrets and applies only the changes that are needed.
///
/// Combines write + diff + selective apply into a single IPC call:
/// 1. Writes config.toml and secrets.toml to disk
/// 2. Diffs old (from state) vs new (from params) in-memory — no re-read
/// 3. Selectively applies changes based on the diff
/// 4. Returns the effective locale for frontend sync
#[tauri::command]
pub async fn save_and_apply(
    config: AppConfig,
    secrets: SecretsConfig,
    app_handle: AppHandle,
) -> Result<SaveResult, CommandError> {
    let state = app_handle.state::<AppState>();

    // Snapshot old config + secrets from state
    let (old_config, old_secrets) = {
        let inner = state.inner.read().await;
        (inner.config.clone(), inner.secrets.clone())
    };

    // Write both files to disk
    config.save()?;
    secrets.save()?;
    tracing::info!("Configuration and secrets saved to disk");

    // Compute diff in-memory (no disk re-read)
    let diff = crate::config_diff::compute(&old_config, &config, &old_secrets, &secrets);

    if diff.is_empty() {
        tracing::info!("Config unchanged, nothing to apply");
        return Ok(SaveResult {
            effective_locale: i18n::get_current_locale(),
        });
    }

    tracing::info!(
        locale_changed = diff.locale_changed,
        autostart_changed = diff.autostart_changed,
        needs_registry_rebuild = diff.needs_registry_rebuild,
        games_to_refresh = ?diff.games_to_refresh,
        games_to_reset_notifications = ?diff.games_to_reset_notifications,
        "Config diff computed"
    );

    // Apply new config + secrets, optionally rebuilding registries
    state
        .apply_config(config, secrets, diff.needs_registry_rebuild)
        .await;

    // Update locale if changed
    if diff.locale_changed {
        let language = {
            let inner = state.inner.read().await;
            inner.config.general.language.clone()
        };
        let effective_locale = i18n::resolve_locale(language.as_deref());
        if let Err(e) = i18n::set_locale(effective_locale) {
            tracing::warn!(error = %e, "Failed to update i18n locale");
        }
        if let Err(e) = crate::tray::build_tray_menu(&app_handle) {
            tracing::warn!(error = %e, "Failed to rebuild tray menu");
        }
    }

    // Sync autostart if changed
    if diff.autostart_changed {
        let autostart_enabled = {
            let inner = state.inner.read().await;
            inner.config.general.autostart
        };
        let autolaunch = app_handle.autolaunch();
        let autostart_result = if autostart_enabled {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
        if let Err(e) = autostart_result {
            tracing::warn!(error = %e, "Failed to sync autostart state");
        }
    }

    // Reset notification cooldowns for affected games only
    if !diff.games_to_reset_notifications.is_empty() {
        let mut inner = state.inner.write().await;
        for game_id in &diff.games_to_reset_notifications {
            inner.notification_tracker.clear_for_game(*game_id);
        }
    }

    // Selective refresh: only fetch games that actually changed
    if !diff.games_to_refresh.is_empty() {
        let _ = polling::refresh_games(&app_handle, &diff.games_to_refresh).await;
    }

    tracing::info!("Configuration applied successfully");
    Ok(SaveResult {
        effective_locale: i18n::get_current_locale(),
    })
}

/// Opens the configuration folder in the system file manager.
#[tauri::command]
pub fn open_config_folder() -> Result<(), CommandError> {
    let config_dir = storekeeper_core::AppConfig::config_dir()?;

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }

    // Open in file manager
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&config_dir)
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&config_dir)
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&config_dir)
            .spawn()?;
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
) -> Result<AllDailyRewardStatus, CommandError> {
    Ok(state.get_daily_reward_status().await)
}

/// Refreshes the daily reward status from all configured games.
#[tauri::command]
pub async fn refresh_daily_reward_status(
    state: State<'_, AppState>,
) -> Result<AllDailyRewardStatus, CommandError> {
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
) -> Result<HashMap<GameId, serde_json::Value>, CommandError> {
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
) -> Result<serde_json::Value, CommandError> {
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
) -> Result<serde_json::Value, CommandError> {
    Ok(state.get_daily_reward_status_for_game(game_id).await?)
}

// ============================================================================
// Notification Commands
// ============================================================================

/// Sends a preview notification for a specific game resource using cached data.
#[tauri::command]
pub async fn send_preview_notification(
    game_id: GameId,
    resource_type: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let resources = state.get_resources().await;
    let game_name = notification::game_display_name(game_id);
    let resource_name = notification::resource_display_name(&resource_type);

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
            let info = notification::extract_resource_info(&resource_type, data)?;
            let now = Utc::now();
            Some(notification::build_notification_body(&info, now))
        })
        .unwrap_or_else(|| i18n::t("notification_no_data"));

    let title = i18n::t_args(
        "notification_title",
        &[
            ("game_name", i18n::Value::from(game_name.as_str())),
            ("resource_name", i18n::Value::from(resource_name.as_str())),
        ],
    );

    app_handle
        .notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| CommandError {
            code: ErrorCode::NotificationError,
            message: e.to_string(),
        })
}

// ============================================================================
// Locale Commands
// ============================================================================

/// Returns the list of supported locale codes.
#[tauri::command]
pub fn get_supported_locales() -> Vec<&'static str> {
    i18n::supported_locales()
}

/// Returns the effective locale currently in use by the backend.
#[tauri::command]
pub fn get_effective_locale() -> String {
    i18n::get_current_locale()
}
