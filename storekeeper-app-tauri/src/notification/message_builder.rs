//! Notification message building and display name resolution.

use chrono::{DateTime, Local, Timelike, Utc};
use storekeeper_core::GameId;

use crate::i18n;

use super::resource_extractor::ResourceInfo;

/// Maps resource type tags to localized display names via i18n lookup.
pub(crate) fn resource_display_name(game_id: GameId, resource_type: &str) -> String {
    let key = format!("game.{}.resource.{resource_type}", game_id.short_id());
    let result = i18n::t(&key);
    if result == key {
        i18n::t("resource.unknown")
    } else {
        result
    }
}

/// Returns the localized game display name via i18n lookup.
pub(crate) fn game_display_name(game_id: GameId) -> String {
    let key = format!("game.{}.name", game_id.short_id());
    i18n::t(&key)
}

/// Builds the notification body text for a resource.
///
/// Differentiates between stamina resources (have `max`) and cooldown/expedition
/// resources (no `max`). Stamina resources show current/max + duration + clock time;
/// cooldown resources show "ready" or "ready in {duration}".
#[allow(clippy::cast_possible_wrap)]
pub(crate) fn build_notification_body(
    resource_name: &str,
    info: &ResourceInfo,
    now: DateTime<Utc>,
) -> String {
    let is_stamina = info.max.is_some();

    if is_stamina {
        if info.is_complete {
            return i18n::t_args(
                "notification.resource_full",
                &[("resource_name", i18n::Value::from(resource_name))],
            );
        }

        let current = info
            .estimated_current(now)
            .map_or_else(|| "?".to_string(), |v| v.to_string());
        let max = info.max.map_or_else(|| "?".to_string(), |v| v.to_string());
        let mins_remaining = (info.completion_at - now).num_minutes();
        let duration = i18n::format_duration(mins_remaining);
        let local_time = info.completion_at.with_timezone(&Local);
        let hour = u8::try_from(local_time.hour()).unwrap_or(0);
        let minute = u8::try_from(local_time.minute()).unwrap_or(0);
        let clock_time = i18n::format_time(hour, minute);

        i18n::t_args(
            "notification.resource_status",
            &[
                ("resource_name", i18n::Value::from(resource_name)),
                ("current", i18n::Value::from(current)),
                ("max", i18n::Value::from(max)),
                ("duration", i18n::Value::from(duration)),
                ("clock_time", i18n::Value::from(clock_time)),
            ],
        )
    } else {
        if info.is_complete {
            return i18n::t_args(
                "notification.resource_ready",
                &[("resource_name", i18n::Value::from(resource_name))],
            );
        }

        let mins_remaining = (info.completion_at - now).num_minutes();
        let duration = i18n::format_duration(mins_remaining);

        i18n::t_args(
            "notification.resource_ready_in",
            &[
                ("resource_name", i18n::Value::from(resource_name)),
                ("duration", i18n::Value::from(duration)),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;
    use crate::notification::resource_extractor::ResourceInfo;

    /// Ensures i18n is initialized for tests.
    fn ensure_init() {
        let _ = crate::i18n::init("en");
    }

    // =========================================================================
    // resource_display_name tests
    // =========================================================================

    #[test]
    fn test_display_names() {
        ensure_init();
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
        ensure_init();
        assert_eq!(
            resource_display_name(GameId::GenshinImpact, "unknown_thing"),
            "Unknown Resource"
        );
    }

    // =========================================================================
    // body text tests — stamina resources
    // =========================================================================

    #[test]
    fn test_stamina_full() {
        ensure_init();
        let now = Utc::now();
        let info = ResourceInfo {
            completion_at: now,
            is_complete: true,
            current: Some(160),
            max: Some(160),
            regen_rate_seconds: Some(480),
        };
        let body = build_notification_body("Original Resin", &info, now);
        assert_eq!(body, "Original Resin is full!");
    }

    #[test]
    fn test_stamina_not_full_shows_status() {
        ensure_init();
        let now = Utc::now();
        let completion_at = now + TimeDelta::minutes(75);
        let info = ResourceInfo {
            completion_at,
            is_complete: false,
            current: Some(140),
            max: Some(160),
            regen_rate_seconds: Some(480),
        };
        let body = build_notification_body("Original Resin", &info, now);
        // Should contain resource name, current/max, and time info
        assert!(body.contains("Original Resin"));
        assert!(body.contains("/160"));
    }

    // =========================================================================
    // body text tests — cooldown resources
    // =========================================================================

    #[test]
    fn test_cooldown_complete() {
        ensure_init();
        let now = Utc::now();
        let info = ResourceInfo {
            completion_at: now,
            is_complete: true,
            current: None,
            max: None,
            regen_rate_seconds: None,
        };
        let body = build_notification_body("Parametric Transformer", &info, now);
        assert_eq!(body, "Parametric Transformer is ready to claim!");
    }

    #[test]
    fn test_cooldown_not_complete() {
        ensure_init();
        let now = Utc::now();
        let completion_at = now + TimeDelta::minutes(30);
        let info = ResourceInfo {
            completion_at,
            is_complete: false,
            current: None,
            max: None,
            regen_rate_seconds: None,
        };
        let body = build_notification_body("Parametric Transformer", &info, now);
        assert!(body.contains("Parametric Transformer"));
        assert!(body.contains("will be ready in"));
    }
}
