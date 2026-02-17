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
use storekeeper_core::GameId;
use storekeeper_core::config::{GamesConfig, ResourceNotificationConfig};
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

use self::resource_extractor::ResourceInfo;

/// Resolves a resource JSON object into its notification config and extracted
/// timing info, returning `None` if the resource is missing fields, has no
/// config, or notifications are disabled.
fn resolve_notifiable_resource<'a>(
    resource_obj: &'a serde_json::Value,
    games_config: &'a GamesConfig,
    game_id: GameId,
) -> Option<(&'a str, &'a ResourceNotificationConfig, ResourceInfo)> {
    let type_tag = resource_obj
        .get("type")
        .and_then(serde_json::Value::as_str)?;
    let config = games_config.notification_config(game_id, type_tag)?;
    if !config.enabled {
        return None;
    }
    let data = resource_obj.get("data")?;
    let resource_info = extract_resource_info(type_tag, data)?;
    Some((type_tag, config, resource_info))
}

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
    let resources = state.get_resources().await;

    // Snapshot configs so the checker loop does not hold state locks while formatting/sending.
    let games_config = {
        let inner = state.inner.read().await;
        inner.config.games.clone()
    };

    for (game_id, resources_json) in &resources.games {
        if !games_config.has_notification_configs(*game_id) {
            continue;
        }

        let Some(resource_array) = resources_json.as_array() else {
            continue;
        };

        for resource_obj in resource_array {
            let Some((type_tag, config, resource_info)) =
                resolve_notifiable_resource(resource_obj, &games_config, *game_id)
            else {
                continue;
            };

            let should_notify = {
                let mut inner = state.inner.write().await;
                inner.notification_tracker.should_notify(
                    *game_id,
                    type_tag,
                    config,
                    &resource_info,
                    now,
                )
            };

            if !should_notify {
                continue;
            }

            if checker::send_resource_notification(
                app_handle,
                *game_id,
                type_tag,
                &resource_info,
                now,
            ) {
                let mut inner = state.inner.write().await;
                inner.notification_tracker.record(*game_id, type_tag, now);
            }
        }
    }
}
