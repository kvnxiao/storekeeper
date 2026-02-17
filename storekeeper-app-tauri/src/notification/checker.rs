//! Single-resource notification check and OS notification send logic.

use chrono::{DateTime, Utc};
use storekeeper_core::{GameId, ResourceNotificationConfig};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::i18n;

use super::message_builder::{build_notification_body, game_display_name, resource_display_name};
use super::resource_extractor::ResourceInfo;
use super::tracker::NotificationTracker;

/// Checks a single resource against its notification config and sends if needed.
pub(crate) fn check_resource_and_notify(
    app_handle: &AppHandle,
    tracker: &mut NotificationTracker,
    game_id: GameId,
    resource_type: &str,
    info: &ResourceInfo,
    config: &ResourceNotificationConfig,
    now: DateTime<Utc>,
) {
    if !tracker.should_notify(game_id, resource_type, config, info, now) {
        return;
    }

    let game_name = game_display_name(game_id);
    let resource_name = resource_display_name(game_id, resource_type);

    let body = build_notification_body(&resource_name, info, now);

    let title = i18n::t_args(
        "notification.title",
        &[
            ("game_name", i18n::Value::from(game_name.as_str())),
            ("resource_name", i18n::Value::from(resource_name.as_str())),
        ],
    );

    tracing::info!(
        game = game_name.as_str(),
        resource = resource_type,
        body = %body,
        "Sending resource notification"
    );

    let result = app_handle
        .notification()
        .builder()
        .title(&title)
        .body(&body)
        .show();

    match result {
        Ok(()) => {
            tracker.record(game_id, resource_type, now);
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to send notification");
        }
    }
}
