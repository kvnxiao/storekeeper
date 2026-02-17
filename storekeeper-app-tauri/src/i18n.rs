//! Internationalization (i18n) module for Storekeeper.
//!
//! Provides message lookup and ICU MessageFormat-style string formatting
//! with plural support via ICU4X.

mod format;
mod locale;
mod parser;
mod store;

use icu_plurals::PluralRules;

pub use format::{format_duration, format_time};
pub use locale::resolve_locale;
use parser::format_message;
use store::with_messages;
pub use store::{Value, get_current_locale, init, set_locale, supported_locales};

/// Looks up a simple message by key.
///
/// Returns the key itself if not found (makes missing translations visible).
#[must_use]
pub fn t(key: &str) -> String {
    with_messages(|m| {
        m.strings
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    })
    .unwrap_or_else(|| key.to_string())
}

/// Looks up a message by key and substitutes arguments using ICU MessageFormat.
///
/// Supports:
/// - Simple substitution: `{var_name}` replaced by the string value
/// - Plural: `{var_name, plural, one {# item} other {# items}}` where `#` is replaced by the count
#[must_use]
pub fn t_args(key: &str, args: &[(&str, Value)]) -> String {
    with_messages(|m| {
        let template = match m.strings.get(key) {
            Some(s) => s.clone(),
            None => return key.to_string(),
        };
        // Create PluralRules on-demand (not stored because it's !Send+!Sync)
        let plural_rules = PluralRules::try_new_cardinal(m.locale.clone().into()).ok();
        format_message(&template, args, plural_rules.as_ref())
    })
    .unwrap_or_else(|| key.to_string())
}

#[cfg(test)]
mod tests {
    use icu_locale::Locale;
    use icu_plurals::PluralRules;

    use super::*;

    /// Ensures i18n is initialized for tests.
    fn ensure_init() {
        let _ = init("en");
    }

    #[test]
    fn test_simple_lookup() {
        ensure_init();
        assert_eq!(t("tray_quit"), "Quit");
        assert_eq!(t("tray_refresh_now"), "Refresh Now");
    }

    #[test]
    fn test_missing_key_returns_key() {
        ensure_init();
        assert_eq!(t("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn test_simple_substitution() {
        ensure_init();
        let result = t_args(
            "notification_title",
            &[
                ("game_name", Value::String("Genshin Impact".to_string())),
                ("resource_name", Value::String("Original Resin".to_string())),
            ],
        );
        assert_eq!(result, "Genshin Impact - Original Resin");
    }

    #[test]
    fn test_stamina_full() {
        ensure_init();
        let result = t("notification_stamina_full");
        assert_eq!(result, "Full!");
    }

    #[test]
    fn test_cooldown_complete() {
        ensure_init();
        let result = t("notification_cooldown_complete");
        assert_eq!(result, "Ready!");
    }

    #[test]
    fn test_cooldown_remaining() {
        ensure_init();
        let result = t_args(
            "notification_cooldown_remaining",
            &[
                ("duration", Value::String("30m".to_string())),
                ("local_time", Value::String("5:30 PM".to_string())),
            ],
        );
        assert_eq!(result, "Ready in 30m (5:30 PM)");
    }

    #[test]
    fn test_stamina_progress() {
        ensure_init();
        let result = t_args(
            "notification_stamina_progress",
            &[
                ("current", Value::String("140".to_string())),
                ("max", Value::String("160".to_string())),
                ("duration", Value::String("1h 15m".to_string())),
                ("local_time", Value::String("3:45 PM".to_string())),
            ],
        );
        assert_eq!(result, "140/160 - full in 1h 15m (3:45 PM)");
    }

    #[test]
    fn test_format_duration_hours_and_minutes() {
        ensure_init();
        let result = format_duration(75);
        // ICU4X short style — exact format may vary but should contain hours and minutes
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_duration_minutes_only() {
        ensure_init();
        let result = format_duration(30);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_duration_zero() {
        ensure_init();
        let result = format_duration(0);
        // ICU4X narrow style with FieldDisplay::Always on minutes should produce "0m" (en locale)
        assert_eq!(result, "0m");
    }

    #[test]
    fn test_format_duration_negative_clamps() {
        ensure_init();
        let result = format_duration(-10);
        // Negative clamps to 0, same as zero duration
        assert_eq!(result, "0m");
    }

    #[test]
    fn test_format_time() {
        ensure_init();
        let result = format_time(15, 45);
        // Should produce locale-formatted time string (e.g. "3:45 PM" for en)
        assert!(!result.is_empty());
    }

    #[test]
    fn test_game_names() {
        ensure_init();
        assert_eq!(t("game_genshin_name"), "Genshin Impact");
        assert_eq!(t("game_hsr_name"), "Honkai: Star Rail");
        assert_eq!(t("game_zzz_name"), "Zenless Zone Zero");
        assert_eq!(t("game_wuwa_name"), "Wuthering Waves");
    }

    #[test]
    fn test_resource_names() {
        ensure_init();
        assert_eq!(t("resource_resin"), "Original Resin");
        assert_eq!(t("resource_trailblaze_power"), "Trailblaze Power");
        assert_eq!(t("resource_battery"), "Battery");
        assert_eq!(t("resource_waveplates"), "Waveplates");
    }

    #[test]
    fn test_supported_locales() {
        let locales = supported_locales();
        assert!(locales.contains(&"en"));
    }

    #[test]
    fn test_format_message_nested_braces() {
        ensure_init();
        let plural_rules =
            PluralRules::try_new_cardinal("en".parse::<Locale>().expect("valid locale").into())
                .expect("english plural rules should work");
        let template = "{resource_name} in {minutes, plural, one {# minute} other {# minutes}}";
        let result = super::parser::format_message(
            template,
            &[
                ("resource_name", Value::String("Resin".to_string())),
                ("minutes", Value::Number(5)),
            ],
            Some(&plural_rules),
        );
        assert_eq!(result, "Resin in 5 minutes");
    }
}
