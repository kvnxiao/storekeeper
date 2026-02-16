//! Notification message building and display name resolution.

use chrono::{DateTime, Utc};
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
/// Uses i18n-formatted strings based on the resource state.
pub(crate) fn build_notification_body(
    resource_name: &str,
    info: &ResourceInfo,
    is_value_mode: bool,
    now: DateTime<Utc>,
) -> String {
    if info.is_complete {
        let overdue_mins = (now - info.completion_at).num_minutes();
        if overdue_mins <= 0 {
            i18n::t_args(
                "notification.resource_full",
                &[("resource_name", i18n::Value::from(resource_name))],
            )
        } else {
            i18n::t_args(
                "notification.resource_full_overdue",
                &[
                    ("resource_name", i18n::Value::from(resource_name)),
                    ("minutes", i18n::Value::Number(overdue_mins)),
                ],
            )
        }
    } else if is_value_mode {
        match (info.current, info.max) {
            (Some(current), Some(max)) => i18n::t_args(
                "notification.resource_reached",
                &[
                    ("resource_name", i18n::Value::from(resource_name)),
                    (
                        "current",
                        i18n::Value::Number(i64::try_from(current).unwrap_or(i64::MAX)),
                    ),
                    (
                        "max",
                        i18n::Value::Number(i64::try_from(max).unwrap_or(i64::MAX)),
                    ),
                ],
            ),
            _ => i18n::t_args(
                "notification.resource_threshold_reached",
                &[("resource_name", i18n::Value::from(resource_name))],
            ),
        }
    } else {
        let mins_remaining = (info.completion_at - now).num_minutes();
        i18n::t_args(
            "notification.resource_full_in",
            &[
                ("resource_name", i18n::Value::from(resource_name)),
                ("minutes", i18n::Value::Number(mins_remaining)),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;

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
    // body text tests
    // =========================================================================

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
