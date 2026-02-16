//! Background notification checker for resource completion alerts.
//!
//! Runs on a 60-second timer, reads cached resources from state, and sends
//! OS notifications when resources are approaching full or have been full.

mod checker;
mod message_builder;
mod resource_extractor;
mod tracker;

// Re-export public/pub(crate) items so they remain accessible at `notification::*`.
pub(crate) use message_builder::{
    build_notification_body, game_display_name, resource_display_name,
};
pub(crate) use resource_extractor::extract_resource_info;
pub use tracker::NotificationTracker;

use chrono::Utc;
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

/// Starts the background notification checker.
///
/// Runs every 60 seconds, checking cached resources against per-game
/// notification thresholds. Does not make API calls — reads state only.
pub fn start_notification_checker(app_handle: AppHandle, cancel_token: CancellationToken) {
    tauri::async_runtime::spawn(async move {
        tracing::info!("Starting notification checker task");

        loop {
            tokio::select! {
                () = cancel_token.cancelled() => {
                    tracing::info!("Notification checker cancelled");
                    break;
                }
                () = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                    check_and_notify(&app_handle).await;
                }
            }
        }
    });
}

/// Checks all cached resources against notification thresholds.
pub(crate) async fn check_and_notify(app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();
    let now = Utc::now();

    // Read resources + notification configs (read lock, released after this block)
    let resources = state.get_resources().await;
    let mut game_configs = Vec::new();
    for (game_id, resources_json) in &resources.games {
        let configs = state.get_game_notification_config(*game_id).await;
        if !configs.is_empty() {
            game_configs.push((*game_id, configs, resources_json));
        }
    }

    // Single write lock for all tracker mutations
    let mut inner = state.inner.write().await;
    for (game_id, notification_configs, resources_json) in &game_configs {
        let Some(resource_array) = resources_json.as_array() else {
            continue;
        };

        for resource_obj in resource_array {
            let Some(type_tag) = resource_obj.get("type").and_then(serde_json::Value::as_str)
            else {
                continue;
            };

            let Some(config) = notification_configs.get(type_tag) else {
                continue;
            };

            if !config.enabled {
                continue;
            }

            let Some(data) = resource_obj.get("data") else {
                continue;
            };

            let Some(resource_info) = extract_resource_info(data) else {
                continue;
            };

            checker::check_resource_and_notify(
                app_handle,
                &mut inner.notification_tracker,
                *game_id,
                type_tag,
                &resource_info,
                config,
                now,
            );
        }
    }
}
