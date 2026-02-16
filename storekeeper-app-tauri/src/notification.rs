//! Background notification checker for resource completion alerts.
//!
//! Runs on a 60-second timer, reads cached resources from state, and sends
//! OS notifications when resources are approaching full or have been full.

use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc};
use storekeeper_core::{GameId, ResourceNotificationConfig};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

/// Tracks notification cooldown state per (game, resource) pair.
#[derive(Default)]
pub struct NotificationTracker {
    cooldowns: HashMap<(GameId, String), DateTime<Utc>>,
}

impl NotificationTracker {
    /// Decides whether a notification should fire for this resource.
    ///
    /// Returns `false` (and clears cooldown) when the resource is outside the
    /// notification window. Returns `false` when still within cooldown. Returns
    /// `true` when the resource is in-window/full and cooldown has expired or
    /// no prior notification exists.
    ///
    /// When `cooldown_minutes` is 0, only one notification fires per window
    /// entry — no recurring reminders until the resource leaves and re-enters.
    pub fn should_notify(
        &mut self,
        game_id: GameId,
        resource_type: &str,
        config: &ResourceNotificationConfig,
        info: &ResourceInfo,
        now: DateTime<Utc>,
    ) -> bool {
        let in_window = match (config.notify_at_value, config.notify_minutes_before_full) {
            // Value-threshold mode: convert to minutes via regen rate, fallback to direct comparison
            (Some(threshold), _) => {
                if let (Some(max), Some(rate)) = (info.max, info.regen_rate_seconds) {
                    let units_remaining = max.saturating_sub(threshold);
                    let effective_minutes =
                        i64::try_from(units_remaining * rate / 60).unwrap_or(i64::MAX);
                    let time_to_full = info.completion_at - now;
                    info.is_complete || time_to_full <= TimeDelta::minutes(effective_minutes)
                } else {
                    // Fallback: direct value comparison (no rate available)
                    info.current.is_some_and(|c| c >= threshold) || info.is_complete
                }
            }
            // Minutes-before-full mode (existing behavior)
            (None, Some(minutes)) => {
                let threshold = TimeDelta::minutes(i64::from(minutes));
                let time_to_full = info.completion_at - now;
                info.is_complete || time_to_full <= threshold
            }
            // Neither set: notify only when full/ready
            (None, None) => info.is_complete,
        };

        // Not in notification window yet — reset cooldown tracking
        if !in_window {
            self.clear(game_id, resource_type);
            return false;
        }

        // In window or already full — check cooldown
        let key = (game_id, resource_type.to_string());
        if let Some(last_notified) = self.cooldowns.get(&key).copied() {
            // cooldown_minutes == 0 means "notify once, don't repeat"
            if config.cooldown_minutes == 0 {
                return false;
            }
            let cooldown = TimeDelta::minutes(i64::from(config.cooldown_minutes));
            if (now - last_notified) < cooldown {
                return false;
            }
        }

        true
    }

    /// Records that a notification was sent now for this (game, resource) pair.
    pub fn record(&mut self, game_id: GameId, resource_type: &str, now: DateTime<Utc>) {
        self.cooldowns
            .insert((game_id, resource_type.to_string()), now);
    }

    /// Clears the cooldown entry for this (game, resource) pair.
    fn clear(&mut self, game_id: GameId, resource_type: &str) {
        self.cooldowns.remove(&(game_id, resource_type.to_string()));
    }

    /// Clears all cooldown entries. Called on config reload so stale cooldowns
    /// from a previous configuration don't suppress notifications.
    pub fn clear_all(&mut self) {
        self.cooldowns.clear();
    }
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
async fn check_and_notify(app_handle: &AppHandle) {
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

            check_resource_and_notify(
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

/// Extracted timing info from a resource JSON object.
pub(crate) struct ResourceInfo {
    /// When the resource will be complete/full/ready.
    pub(crate) completion_at: DateTime<Utc>,
    /// Whether the resource is already complete.
    pub(crate) is_complete: bool,
    /// Current resource value (stamina resources only).
    pub(crate) current: Option<u64>,
    /// Maximum resource value (stamina resources only).
    pub(crate) max: Option<u64>,
    /// Seconds per unit of regeneration (stamina resources only).
    pub(crate) regen_rate_seconds: Option<u64>,
}

/// Extracts completion timing from a resource data object.
///
/// Detects resource kind by field presence:
/// - Has `fullAt` + `current` + `max` → `StaminaResource`
/// - Has `readyAt` + `isReady` → `CooldownResource`
/// - Has `earliestFinishAt` → `ExpeditionResource`
pub(crate) fn extract_resource_info(data: &serde_json::Value) -> Option<ResourceInfo> {
    // StaminaResource: fullAt, current, max
    if let Some(full_at_str) = data.get("fullAt").and_then(serde_json::Value::as_str) {
        let completion_at = DateTime::parse_from_rfc3339(full_at_str)
            .or_else(|_| DateTime::parse_from_str(full_at_str, "%Y-%m-%dT%H:%M:%S%.f%:z"))
            .ok()?
            .with_timezone(&Utc);

        let current = data.get("current").and_then(serde_json::Value::as_u64);
        let max = data.get("max").and_then(serde_json::Value::as_u64);
        let regen_rate_seconds = data
            .get("regenRateSeconds")
            .and_then(serde_json::Value::as_u64);

        let is_complete = match (current, max) {
            (Some(c), Some(m)) if m > 0 => c >= m,
            _ => false,
        };

        return Some(ResourceInfo {
            completion_at,
            is_complete,
            current,
            max,
            regen_rate_seconds,
        });
    }

    // CooldownResource: readyAt, isReady
    if let Some(ready_at_str) = data.get("readyAt").and_then(serde_json::Value::as_str) {
        let completion_at = DateTime::parse_from_rfc3339(ready_at_str)
            .or_else(|_| DateTime::parse_from_str(ready_at_str, "%Y-%m-%dT%H:%M:%S%.f%:z"))
            .ok()?
            .with_timezone(&Utc);

        let is_ready = data
            .get("isReady")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        return Some(ResourceInfo {
            completion_at,
            is_complete: is_ready,
            current: None,
            max: None,
            regen_rate_seconds: None,
        });
    }

    // ExpeditionResource: earliestFinishAt
    if let Some(finish_at_str) = data
        .get("earliestFinishAt")
        .and_then(serde_json::Value::as_str)
    {
        let completion_at = DateTime::parse_from_rfc3339(finish_at_str)
            .or_else(|_| DateTime::parse_from_str(finish_at_str, "%Y-%m-%dT%H:%M:%S%.f%:z"))
            .ok()?
            .with_timezone(&Utc);

        let is_complete = completion_at <= Utc::now();

        return Some(ResourceInfo {
            completion_at,
            is_complete,
            current: None,
            max: None,
            regen_rate_seconds: None,
        });
    }

    None
}

/// Checks a single resource against its notification config and sends if needed.
fn check_resource_and_notify(
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

    let game_name = game_id.display_name();
    let resource_name = resource_display_name(game_id, resource_type);

    let time_to_full = info.completion_at - now;
    let body = if info.is_complete {
        let overdue_mins = (now - info.completion_at).num_minutes();
        if overdue_mins <= 0 {
            format!("{resource_name} is full!")
        } else {
            format!("{resource_name} has been full for {overdue_mins} minutes")
        }
    } else if config.notify_at_value.is_some() {
        match (info.current, info.max) {
            (Some(current), Some(max)) => {
                format!("{resource_name} has reached {current}/{max}")
            }
            _ => format!("{resource_name} threshold reached"),
        }
    } else {
        let mins_remaining = time_to_full.num_minutes();
        format!("{resource_name} will be full in {mins_remaining} minutes")
    };

    tracing::info!(
        game = game_name,
        resource = resource_type,
        body = %body,
        "Sending resource notification"
    );

    let result = app_handle
        .notification()
        .builder()
        .title(format!("{game_name} — {resource_name}"))
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

/// Maps resource type tags to human-readable display names.
pub(crate) fn resource_display_name(game_id: GameId, resource_type: &str) -> &'static str {
    match (game_id, resource_type) {
        // Genshin Impact
        (GameId::GenshinImpact, "resin") => "Original Resin",
        (GameId::GenshinImpact, "parametric_transformer") => "Parametric Transformer",
        (GameId::GenshinImpact, "realm_currency") => "Realm Currency",
        (GameId::GenshinImpact, "expeditions") => "Expeditions",
        // Honkai: Star Rail
        (GameId::HonkaiStarRail, "trailblaze_power") => "Trailblaze Power",
        // Zenless Zone Zero
        (GameId::ZenlessZoneZero, "battery") => "Battery",
        // Wuthering Waves
        (GameId::WutheringWaves, "waveplates") => "Waveplates",
        // Fallback: return a leaked string from the type tag
        // (acceptable since this is a bounded set of known resource types)
        _ => "Unknown Resource",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // extract_resource_info tests
    // =========================================================================

    #[test]
    fn test_extract_stamina_resource() {
        let future = Utc::now() + TimeDelta::hours(2);
        let data = serde_json::json!({
            "current": 100,
            "max": 160,
            "fullAt": future.to_rfc3339(),
            "regenRateSeconds": 480
        });

        let info = extract_resource_info(&data).expect("should extract stamina resource");
        assert!(!info.is_complete);
        assert!((info.completion_at - future).num_seconds().abs() < 2);
    }

    #[test]
    fn test_extract_stamina_resource_full() {
        let past = Utc::now() - TimeDelta::hours(1);
        let data = serde_json::json!({
            "current": 160,
            "max": 160,
            "fullAt": past.to_rfc3339(),
            "regenRateSeconds": 480
        });

        let info = extract_resource_info(&data).expect("should extract full stamina resource");
        assert!(info.is_complete);
    }

    #[test]
    fn test_extract_cooldown_resource_ready() {
        let past = Utc::now() - TimeDelta::hours(1);
        let data = serde_json::json!({
            "isReady": true,
            "readyAt": past.to_rfc3339()
        });

        let info = extract_resource_info(&data).expect("should extract cooldown resource");
        assert!(info.is_complete);
    }

    #[test]
    fn test_extract_cooldown_resource_not_ready() {
        let future = Utc::now() + TimeDelta::hours(12);
        let data = serde_json::json!({
            "isReady": false,
            "readyAt": future.to_rfc3339()
        });

        let info = extract_resource_info(&data).expect("should extract cooldown resource");
        assert!(!info.is_complete);
    }

    #[test]
    fn test_extract_expedition_resource_completed() {
        let past = Utc::now() - TimeDelta::minutes(30);
        let data = serde_json::json!({
            "currentExpeditions": 3,
            "maxExpeditions": 5,
            "earliestFinishAt": past.to_rfc3339()
        });

        let info = extract_resource_info(&data).expect("should extract expedition resource");
        assert!(info.is_complete);
    }

    #[test]
    fn test_extract_expedition_resource_pending() {
        let future = Utc::now() + TimeDelta::hours(6);
        let data = serde_json::json!({
            "currentExpeditions": 3,
            "maxExpeditions": 5,
            "earliestFinishAt": future.to_rfc3339()
        });

        let info = extract_resource_info(&data).expect("should extract expedition resource");
        assert!(!info.is_complete);
    }

    #[test]
    fn test_extract_unknown_resource_returns_none() {
        let data = serde_json::json!({
            "someUnknownField": 42
        });

        assert!(extract_resource_info(&data).is_none());
    }

    // =========================================================================
    // resource_display_name tests
    // =========================================================================

    #[test]
    fn test_display_names() {
        assert_eq!(
            resource_display_name(GameId::GenshinImpact, "resin"),
            "Original Resin"
        );
        assert_eq!(
            resource_display_name(GameId::GenshinImpact, "parametric_transformer"),
            "Parametric Transformer"
        );
        assert_eq!(
            resource_display_name(GameId::GenshinImpact, "realm_currency"),
            "Realm Currency"
        );
        assert_eq!(
            resource_display_name(GameId::GenshinImpact, "expeditions"),
            "Expeditions"
        );
        assert_eq!(
            resource_display_name(GameId::HonkaiStarRail, "trailblaze_power"),
            "Trailblaze Power"
        );
        assert_eq!(
            resource_display_name(GameId::ZenlessZoneZero, "battery"),
            "Battery"
        );
        assert_eq!(
            resource_display_name(GameId::WutheringWaves, "waveplates"),
            "Waveplates"
        );
    }

    #[test]
    fn test_display_name_unknown_fallback() {
        assert_eq!(
            resource_display_name(GameId::GenshinImpact, "unknown_thing"),
            "Unknown Resource"
        );
    }

    // =========================================================================
    // NotificationTracker tests
    // =========================================================================

    fn stub_config(threshold_min: u32, cooldown_min: u32) -> ResourceNotificationConfig {
        ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: if threshold_min > 0 {
                Some(threshold_min)
            } else {
                None
            },
            notify_at_value: None,
            cooldown_minutes: cooldown_min,
        }
    }

    fn stub_info(completion_at: DateTime<Utc>, is_complete: bool) -> ResourceInfo {
        ResourceInfo {
            completion_at,
            is_complete,
            current: None,
            max: None,
            regen_rate_seconds: None,
        }
    }

    #[test]
    fn test_not_in_window_clears_and_returns_false() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let game = GameId::GenshinImpact;
        let config = stub_config(60, 10);

        // Seed a prior cooldown entry
        tracker.record(game, "resin", now - TimeDelta::hours(1));

        let info = stub_info(now + TimeDelta::hours(2), false);
        assert!(!tracker.should_notify(game, "resin", &config, &info, now));

        // Internal state was cleared — next in-window check should return true
        let in_window_info = stub_info(now + TimeDelta::minutes(30), false);
        assert!(tracker.should_notify(game, "resin", &config, &in_window_info, now));
    }

    #[test]
    fn test_in_window_first_time_returns_true() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let config = stub_config(60, 10);
        let info = stub_info(now + TimeDelta::minutes(30), false);

        assert!(tracker.should_notify(GameId::GenshinImpact, "resin", &config, &info, now));
    }

    #[test]
    fn test_in_window_within_cooldown_returns_false() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let game = GameId::GenshinImpact;
        let config = stub_config(60, 10);

        tracker.record(game, "resin", now);

        let info = stub_info(now + TimeDelta::minutes(30), false);
        // 5 minutes later, still within 10-minute cooldown
        let later = now + TimeDelta::minutes(5);
        assert!(!tracker.should_notify(game, "resin", &config, &info, later));
    }

    #[test]
    fn test_in_window_after_cooldown_expired_returns_true() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let game = GameId::HonkaiStarRail;
        let config = stub_config(60, 10);

        tracker.record(game, "trailblaze_power", now);

        let info = stub_info(now + TimeDelta::minutes(30), false);
        // 11 minutes later, past 10-minute cooldown
        let later = now + TimeDelta::minutes(11);
        assert!(tracker.should_notify(game, "trailblaze_power", &config, &info, later));
    }

    #[test]
    fn test_at_full_returns_true() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let config = stub_config(60, 10);
        let info = stub_info(now - TimeDelta::seconds(1), true);

        assert!(tracker.should_notify(GameId::ZenlessZoneZero, "battery", &config, &info, now));
    }

    #[test]
    fn test_clear_resets_state() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let game = GameId::WutheringWaves;
        let config = stub_config(60, 10);

        tracker.record(game, "waveplates", now);

        let info = stub_info(now + TimeDelta::minutes(30), false);
        // Within cooldown — should be false
        assert!(!tracker.should_notify(game, "waveplates", &config, &info, now));

        // Manually clear — next check should return true
        tracker.clear(game, "waveplates");
        assert!(tracker.should_notify(game, "waveplates", &config, &info, now));
    }

    #[test]
    fn test_zero_cooldown_notifies_once_then_stops() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let game = GameId::GenshinImpact;
        let config = stub_config(60, 0); // cooldown_minutes = 0

        let info = stub_info(now + TimeDelta::minutes(30), false);

        // First check — no prior notification, should fire
        assert!(tracker.should_notify(game, "resin", &config, &info, now));
        tracker.record(game, "resin", now);

        // Subsequent checks — never re-notifies regardless of time elapsed
        let much_later = now + TimeDelta::hours(24);
        assert!(!tracker.should_notify(game, "resin", &config, &info, much_later));
    }

    #[test]
    fn test_zero_cooldown_resets_on_leaving_window() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let game = GameId::GenshinImpact;
        let config = stub_config(60, 0);

        // In window — notifies
        let in_window = stub_info(now + TimeDelta::minutes(30), false);
        assert!(tracker.should_notify(game, "resin", &config, &in_window, now));
        tracker.record(game, "resin", now);

        // Leaves window (resource consumed) — clears state
        let out_of_window = stub_info(now + TimeDelta::hours(5), false);
        assert!(!tracker.should_notify(game, "resin", &config, &out_of_window, now));

        // Re-enters window — should notify again (one-shot reset)
        assert!(tracker.should_notify(game, "resin", &config, &in_window, now));
    }

    // =========================================================================
    // Value-threshold mode tests
    // =========================================================================

    #[test]
    fn test_value_threshold_with_regen_rate() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: Some(140),
            cooldown_minutes: 10,
        };

        // Resin: max=160, rate=480s/unit. threshold=140, remaining=20 units, 20*480/60=160 min
        let info = ResourceInfo {
            completion_at: now + TimeDelta::minutes(100), // within 160 min window
            is_complete: false,
            current: Some(120),
            max: Some(160),
            regen_rate_seconds: Some(480),
        };

        assert!(tracker.should_notify(GameId::GenshinImpact, "resin", &config, &info, now));
    }

    #[test]
    fn test_value_threshold_not_in_window() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: Some(140),
            cooldown_minutes: 10,
        };

        // threshold=140, remaining=20 units, 20*480/60=160 min. time_to_full=200 > 160
        let info = ResourceInfo {
            completion_at: now + TimeDelta::minutes(200),
            is_complete: false,
            current: Some(100),
            max: Some(160),
            regen_rate_seconds: Some(480),
        };

        assert!(!tracker.should_notify(GameId::GenshinImpact, "resin", &config, &info, now));
    }

    #[test]
    fn test_value_threshold_regen_rate_math_boundary() {
        // Verify exact math: threshold=140, max=160, rate=480s/unit
        // units_remaining = 160-140 = 20, effective_minutes = 20*480/60 = 160
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: Some(140),
            cooldown_minutes: 10,
        };

        // Exactly at boundary (160 min to full) — should notify (<=)
        let at_boundary = ResourceInfo {
            completion_at: now + TimeDelta::minutes(160),
            is_complete: false,
            current: Some(120),
            max: Some(160),
            regen_rate_seconds: Some(480),
        };
        assert!(tracker.should_notify(GameId::GenshinImpact, "resin", &config, &at_boundary, now));

        tracker.clear(GameId::GenshinImpact, "resin");

        // Just outside boundary (161 min to full) — should NOT notify
        let outside_boundary = ResourceInfo {
            completion_at: now + TimeDelta::minutes(161),
            is_complete: false,
            current: Some(119),
            max: Some(160),
            regen_rate_seconds: Some(480),
        };
        assert!(!tracker.should_notify(
            GameId::GenshinImpact,
            "resin",
            &config,
            &outside_boundary,
            now
        ));
    }

    #[test]
    fn test_value_threshold_fallback_direct_comparison() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: Some(140),
            cooldown_minutes: 10,
        };

        // No regen rate — falls back to direct comparison
        let info = ResourceInfo {
            completion_at: now + TimeDelta::hours(1),
            is_complete: false,
            current: Some(145),
            max: Some(160),
            regen_rate_seconds: None,
        };

        assert!(tracker.should_notify(GameId::GenshinImpact, "resin", &config, &info, now));
    }

    #[test]
    fn test_value_threshold_fallback_below_threshold() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: Some(140),
            cooldown_minutes: 10,
        };

        // No regen rate, current < threshold
        let info = ResourceInfo {
            completion_at: now + TimeDelta::hours(1),
            is_complete: false,
            current: Some(100),
            max: Some(160),
            regen_rate_seconds: None,
        };

        assert!(!tracker.should_notify(GameId::GenshinImpact, "resin", &config, &info, now));
    }

    #[test]
    fn test_neither_threshold_only_notifies_when_full() {
        let mut tracker = NotificationTracker::default();
        let now = Utc::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: None,
            cooldown_minutes: 10,
        };

        // Not full — should NOT notify
        let info = stub_info(now + TimeDelta::minutes(5), false);
        assert!(!tracker.should_notify(GameId::GenshinImpact, "resin", &config, &info, now));

        // Full — should notify
        let full_info = stub_info(now - TimeDelta::seconds(1), true);
        assert!(tracker.should_notify(GameId::GenshinImpact, "resin", &config, &full_info, now));
    }

    #[test]
    fn test_body_text_before_full() {
        let now = Utc::now();
        let completion_at = now + TimeDelta::minutes(45);
        let time_to_full = completion_at - now;
        let mins = time_to_full.num_minutes();

        let body = format!("Original Resin will be full in {mins} minutes");
        assert!(body.contains("45 minutes") || body.contains("44 minutes"));
    }

    #[test]
    fn test_body_text_just_full() {
        let now = Utc::now();
        let completion_at = now;
        let overdue = now - completion_at;

        let body = if overdue.num_minutes() <= 0 {
            "Original Resin is full!".to_string()
        } else {
            format!(
                "Original Resin has been full for {} minutes",
                overdue.num_minutes()
            )
        };
        assert_eq!(body, "Original Resin is full!");
    }

    #[test]
    fn test_body_text_after_full() {
        let now = Utc::now();
        let completion_at = now - TimeDelta::minutes(15);
        let overdue = now - completion_at;

        let body = if overdue.num_minutes() <= 0 {
            "Original Resin is full!".to_string()
        } else {
            format!(
                "Original Resin has been full for {} minutes",
                overdue.num_minutes()
            )
        };
        assert!(body.contains("15 minutes"));
    }
}
