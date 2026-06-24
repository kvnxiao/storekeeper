//! Notification message building and display name resolution.

use super::resource_extractor::ResourceInfo;
use crate::i18n;
use jiff::Timestamp;
use jiff::tz::TimeZone;
use storekeeper_core::GameId;

/// Maps resource type tags to localized display names via i18n lookup.
pub(crate) fn resource_display_name(resource_type: &str) -> String {
    let key = format!("resource_{resource_type}");
    let result = i18n::t(&key);
    if result == key {
        i18n::t("resource_unknown")
    } else {
        result
    }
}

/// Returns the localized game display name via i18n lookup.
pub(crate) fn game_display_name(game_id: GameId) -> String {
    let key = format!("game_{}_name", game_id.short_id());
    i18n::t(&key)
}

/// Builds the notification body text for a resource.
///
/// Differentiates between stamina resources (have `max`) and
/// cooldown/expedition resources (no `max`). Stamina resources show
/// current/max, duration, and clock time; cooldown resources show "ready"
/// or "ready in {duration}".
///
/// The resource name is intentionally omitted — the notification title already
/// contains both the game name and resource name.
pub(crate) fn build_notification_body(info: &ResourceInfo, now: Timestamp) -> String {
    let is_stamina = info.max.is_some();

    if is_stamina {
        if info.is_complete {
            return i18n::t("notification_stamina_full");
        }

        let current = info
            .estimated_current(now)
            .map_or_else(|| "?".to_string(), |v| v.to_string());
        let max = info.max.map_or_else(|| "?".to_string(), |v| v.to_string());
        let mins_remaining = info.completion_at.duration_since(now).as_mins();
        let duration = i18n::format_duration(mins_remaining);
        let completion_local = info.completion_at.to_zoned(TimeZone::system());
        let now_local = now.to_zoned(TimeZone::system());
        let local_time = i18n::format_time(&completion_local, &now_local);

        i18n::t_args(
            "notification_stamina_progress",
            &[
                ("current", i18n::Value::from(current)),
                ("max", i18n::Value::from(max)),
                ("duration", i18n::Value::from(duration)),
                ("local_time", i18n::Value::from(local_time)),
            ],
        )
    } else {
        if info.is_complete {
            return i18n::t("notification_cooldown_complete");
        }

        let mins_remaining = info.completion_at.duration_since(now).as_mins();
        let duration = i18n::format_duration(mins_remaining);
        let completion_local = info.completion_at.to_zoned(TimeZone::system());
        let now_local = now.to_zoned(TimeZone::system());
        let local_time = i18n::format_time(&completion_local, &now_local);

        i18n::t_args(
            "notification_cooldown_remaining",
            &[
                ("duration", i18n::Value::from(duration)),
                ("local_time", i18n::Value::from(local_time)),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::resource_extractor::ResourceInfo;
    use jiff::SignedDuration;

    /// Ensures i18n is initialized for tests.
    fn ensure_init() {
        #[expect(
            clippy::let_underscore_must_use,
            reason = "test setup may run after i18n is already initialized"
        )]
        let _ = crate::i18n::init("en");
    }

    // =========================================================================
    // resource_display_name tests
    // =========================================================================

    #[test]
    fn test_display_names() {
        ensure_init();
        assert_eq!(resource_display_name("resin"), "Original Resin");
        assert_eq!(
            resource_display_name("parametric_transformer"),
            "Parametric Transformer"
        );
        assert_eq!(resource_display_name("realm_currency"), "Realm Currency");
        assert_eq!(resource_display_name("expeditions"), "Expeditions");
        assert_eq!(
            resource_display_name("trailblaze_power"),
            "Trailblaze Power"
        );
        assert_eq!(resource_display_name("battery"), "Battery");
        assert_eq!(resource_display_name("waveplates"), "Waveplates");
    }

    #[test]
    fn test_display_name_unknown_fallback() {
        ensure_init();
        assert_eq!(resource_display_name("unknown_thing"), "Unknown Resource");
    }

    // =========================================================================
    // body text tests — stamina resources
    // =========================================================================

    #[test]
    fn test_stamina_full() {
        ensure_init();
        let now = Timestamp::now();
        let info = ResourceInfo {
            completion_at: now,
            is_complete: true,
            current: Some(160),
            max: Some(160),
            regen_rate_seconds: Some(480),
        };
        let body = build_notification_body(&info, now);
        assert_eq!(body, "Full!");
    }

    #[test]
    fn test_stamina_not_full_shows_status() {
        ensure_init();
        let now = Timestamp::now();
        let completion_at = now + SignedDuration::from_mins(75);
        let info = ResourceInfo {
            completion_at,
            is_complete: false,
            current: Some(140),
            max: Some(160),
            regen_rate_seconds: Some(480),
        };
        let body = build_notification_body(&info, now);
        // Should contain /max and time info
        assert!(body.contains("/160"));
    }

    // =========================================================================
    // body text tests — cooldown resources
    // =========================================================================

    #[test]
    fn test_cooldown_complete() {
        ensure_init();
        let now = Timestamp::now();
        let info = ResourceInfo {
            completion_at: now,
            is_complete: true,
            current: None,
            max: None,
            regen_rate_seconds: None,
        };
        let body = build_notification_body(&info, now);
        assert_eq!(body, "Ready!");
    }

    #[test]
    fn test_cooldown_not_complete() {
        ensure_init();
        let now = Timestamp::now();
        let completion_at = now + SignedDuration::from_mins(30);
        let info = ResourceInfo {
            completion_at,
            is_complete: false,
            current: None,
            max: None,
            regen_rate_seconds: None,
        };
        let body = build_notification_body(&info, now);
        assert!(body.contains("Ready in"));
    }

    // =========================================================================
    // duration >= 24h tests
    // =========================================================================

    #[test]
    fn test_stamina_duration_over_24h() {
        ensure_init();
        let now = Timestamp::now();
        // 3000 minutes = 2d 2h 0m
        let completion_at = now + SignedDuration::from_mins(3000);
        let info = ResourceInfo {
            completion_at,
            is_complete: false,
            current: Some(10),
            max: Some(160),
            regen_rate_seconds: Some(480),
        };
        let body = build_notification_body(&info, now);
        // Should contain day unit (e.g. "2d" in narrow format)
        assert!(body.contains("2d"), "Expected '2d' in: {body}");
    }

    #[test]
    fn test_cooldown_duration_over_24h() {
        ensure_init();
        let now = Timestamp::now();
        // 3000 minutes = 2d 2h 0m
        let completion_at = now + SignedDuration::from_mins(3000);
        let info = ResourceInfo {
            completion_at,
            is_complete: false,
            current: None,
            max: None,
            regen_rate_seconds: None,
        };
        let body = build_notification_body(&info, now);
        assert!(body.contains("2d"), "Expected '2d' in: {body}");
    }

    // =========================================================================
    // weekday shown for different-day completion
    // =========================================================================

    #[test]
    fn test_completion_different_day_shows_weekday() {
        ensure_init();
        let now = Timestamp::now();
        let completion_at = now + SignedDuration::from_hours(48);
        let info = ResourceInfo {
            completion_at,
            is_complete: false,
            current: None,
            max: None,
            regen_rate_seconds: None,
        };
        let body = build_notification_body(&info, now);
        let has_weekday = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
            .iter()
            .any(|day| body.contains(day));
        assert!(has_weekday, "Expected weekday in: {body}");
    }
}
