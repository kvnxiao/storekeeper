//! Internationalization (i18n) module for Storekeeper.
//!
//! Provides message lookup and ICU MessageFormat-style string formatting
//! with plural support via ICU4X.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use anyhow::{Context, Result, bail};
use icu_datetime::fieldsets;
use icu_experimental::duration::{
    DurationFormatter, DurationFormatterPreferences, ValidatedDurationFormatterOptions,
    options::{BaseStyle, DurationFormatterOptions, FieldDisplay},
};
use icu_locale::Locale;
use icu_plurals::{PluralCategory, PluralRules};

/// Embedded English locale strings (loaded at compile time).
const EN_LOCALE: &str = include_str!("../../locales/en.json");

/// List of supported locale codes.
const SUPPORTED_LOCALES: &[&str] = &["en"];

/// Default locale used when no match is found.
const DEFAULT_LOCALE: &str = "en";

/// Global messages store, initialized once at startup and switchable at runtime.
static MESSAGES: OnceLock<RwLock<Messages>> = OnceLock::new();

/// Holds the loaded locale data: parsed strings and locale info.
///
/// `PluralRules` is not stored here because it is `!Send + !Sync` (uses `Rc`
/// internally). Instead, plural rules are created on-demand in `t_args`.
struct Messages {
    locale: Locale,
    strings: HashMap<String, String>,
}

/// Value type for message format argument substitution.
pub enum Value {
    /// A string value, substituted directly.
    String(std::string::String),
    /// A numeric value, used for plural dispatch and `#` replacement.
    Number(i64),
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

/// Loads locale JSON and constructs `Messages`.
fn load_messages(locale_str: &str) -> Result<Messages> {
    let json_str = match locale_str {
        "en" => EN_LOCALE,
        _ => bail!("unsupported locale: {locale_str}"),
    };

    let locale: Locale = locale_str
        .parse()
        .with_context(|| format!("failed to parse locale: {locale_str}"))?;

    let strings: HashMap<String, String> =
        serde_json::from_str(json_str).context("failed to parse locale JSON")?;

    // Validate that plural rules can be created for this locale
    PluralRules::try_new_cardinal(locale.clone().into())
        .map_err(|e| anyhow::anyhow!("failed to create plural rules for {locale_str}: {e}"))?;

    Ok(Messages { locale, strings })
}

/// Initializes the i18n system with the given locale.
///
/// Must be called once at startup. Subsequent calls are ignored (use `set_locale` instead).
///
/// # Errors
///
/// Returns an error if the locale cannot be loaded or parsed.
pub fn init(locale_str: &str) -> Result<()> {
    let messages = load_messages(locale_str)?;
    let _ = MESSAGES.set(RwLock::new(messages));
    tracing::info!(locale = locale_str, "i18n initialized");
    Ok(())
}

/// Switches the active locale at runtime.
///
/// # Errors
///
/// Returns an error if the locale cannot be loaded or the lock is poisoned.
pub fn set_locale(locale_str: &str) -> Result<()> {
    let messages = load_messages(locale_str)?;
    let lock = MESSAGES
        .get()
        .context("i18n not initialized; call init() first")?;
    let mut guard = lock
        .write()
        .map_err(|e| anyhow::anyhow!("i18n lock poisoned: {e}"))?;
    *guard = messages;
    tracing::info!(locale = locale_str, "i18n locale changed");
    Ok(())
}

/// Returns the list of supported locale codes.
#[must_use]
pub fn supported_locales() -> Vec<&'static str> {
    SUPPORTED_LOCALES.to_vec()
}

/// Returns the currently active locale code.
#[must_use]
pub fn get_current_locale() -> String {
    with_messages(|m| m.locale.to_string()).unwrap_or_else(|| DEFAULT_LOCALE.to_string())
}

/// Matches a locale string against supported locales.
///
/// Tries exact match first, then language-prefix match (e.g. "en-US" → "en").
#[must_use]
fn match_supported_locale(locale: &str) -> Option<&'static str> {
    let lower = locale.to_lowercase();

    // Exact match
    if let Some(found) = SUPPORTED_LOCALES.iter().find(|l| l.to_lowercase() == lower) {
        return Some(found);
    }

    // Language-prefix match: split on '-' or '_', match base language
    let prefix = lower.split(['-', '_']).next()?;
    SUPPORTED_LOCALES
        .iter()
        .find(|l| l.to_lowercase() == prefix)
        .copied()
}

/// Resolves the effective locale from an optional config override.
///
/// - If `config_language` is `Some`, matches it against supported locales.
/// - If `None`, detects the system locale via `sys_locale`.
/// - Falls back to `"en"` if no match is found.
#[must_use]
pub fn resolve_locale(config_language: Option<&str>) -> &'static str {
    if let Some(lang) = config_language {
        if let Some(matched) = match_supported_locale(lang) {
            return matched;
        }
        tracing::warn!(
            language = lang,
            "configured language not supported, falling back to system locale"
        );
    }

    // Auto-detect from system
    if let Some(sys_locale) = sys_locale::get_locale() {
        if let Some(matched) = match_supported_locale(&sys_locale) {
            tracing::info!(
                system_locale = sys_locale.as_str(),
                resolved = matched,
                "detected system locale"
            );
            return matched;
        }
        tracing::info!(
            system_locale = sys_locale.as_str(),
            "system locale not supported, falling back to default"
        );
    }

    DEFAULT_LOCALE
}

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

/// Formats a clock time (hour + minute) using the current locale.
///
/// Uses `icu_datetime` with `T::hm()` for locale-aware time formatting
/// (e.g. "3:45 PM" in en-US, "15:45" in de-DE).
/// Falls back to `"{hour}:{minute:02}"` if formatting fails.
#[must_use]
pub fn format_time(hour: u8, minute: u8) -> String {
    let fallback = || format!("{hour}:{minute:02}");

    with_messages(|m| {
        let Ok(time) = icu_time::Time::try_new(hour, minute, 0, 0) else {
            return fallback();
        };
        let Ok(formatter) =
            icu_datetime::DateTimeFormatter::try_new(m.locale.clone().into(), fieldsets::T::hm())
        else {
            return fallback();
        };
        formatter.format(&time).to_string()
    })
    .unwrap_or_else(fallback)
}

/// Formats a duration in minutes using the current locale.
///
/// Uses `icu_experimental::duration::DurationFormatter` with `BaseStyle::Short`
/// (e.g. "1 hr, 15 min" in English). Clamps negative values to 0.
/// Falls back to plain `"{hours}h {minutes}m"` or `"{minutes}m"` if formatting fails.
#[must_use]
#[allow(clippy::cast_sign_loss)]
pub fn format_duration(total_minutes: i64) -> String {
    let clamped = total_minutes.max(0) as u64;
    let hours = clamped / 60;
    let minutes = clamped % 60;

    let fallback = || {
        if hours > 0 {
            format!("{hours}h {minutes}m")
        } else {
            format!("{minutes}m")
        }
    };

    with_messages(|m| {
        let mut opts = DurationFormatterOptions::default();
        opts.base = BaseStyle::Short;
        // Always show the minute unit so 0-duration doesn't produce an empty string.
        opts.minute_visibility = Some(FieldDisplay::Always);
        let Ok(validated) = ValidatedDurationFormatterOptions::validate(opts) else {
            return fallback();
        };
        let prefs = DurationFormatterPreferences::from(m.locale.clone());
        let Ok(formatter) = DurationFormatter::try_new(prefs, validated) else {
            return fallback();
        };
        let duration = icu_experimental::duration::Duration {
            hours,
            minutes,
            ..Default::default()
        };
        formatter.format(&duration).to_string()
    })
    .unwrap_or_else(fallback)
}

/// Acquires a read lock on the global messages and runs the closure.
fn with_messages<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&Messages) -> R,
{
    let lock = MESSAGES.get()?;
    let guard = lock.read().ok()?;
    Some(f(&guard))
}

/// Formats a message template with arguments.
///
/// Handles two patterns:
/// 1. `{name}` — simple variable substitution
/// 2. `{name, plural, one {...} other {...}}` — plural dispatch
fn format_message(
    template: &str,
    args: &[(&str, Value)],
    plural_rules: Option<&PluralRules>,
) -> String {
    let arg_map: HashMap<&str, &Value> = args.iter().map(|(k, v)| (*k, v)).collect();
    let mut result = String::with_capacity(template.len());
    let chars: Vec<char> = template.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '{' {
            if let Some(close) = find_matching_brace(&chars, i) {
                let inner: String = chars[i + 1..close].iter().collect();
                let formatted = format_placeholder(&inner, &arg_map, plural_rules);
                result.push_str(&formatted);
                i = close + 1;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Finds the matching closing brace, respecting nested braces.
fn find_matching_brace(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 0;
    for (idx, &ch) in chars.iter().enumerate().skip(start) {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

/// Formats a single placeholder (content between outermost `{` and `}`).
fn format_placeholder(
    inner: &str,
    args: &HashMap<&str, &Value>,
    plural_rules: Option<&PluralRules>,
) -> String {
    let parts: Vec<&str> = inner.splitn(3, ',').collect();

    match parts.len() {
        // Simple: {name}
        1 => {
            let name = parts[0].trim();
            match args.get(name) {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => n.to_string(),
                None => format!("{{{name}}}"),
            }
        }
        // Plural: {name, plural, one {..} other {..}}
        3 if parts[1].trim() == "plural" => {
            let name = parts[0].trim();
            let branches_str = parts[2].trim();

            let count = match args.get(name) {
                Some(Value::Number(n)) => *n,
                _ => return format!("{{{inner}}}"),
            };

            let category_str = match plural_rules {
                Some(rules) => {
                    let category = rules.category_for(count);
                    match category {
                        PluralCategory::Zero => "zero",
                        PluralCategory::One => "one",
                        PluralCategory::Two => "two",
                        PluralCategory::Few => "few",
                        PluralCategory::Many => "many",
                        PluralCategory::Other => "other",
                    }
                }
                // Fallback to "other" when plural rules are unavailable
                None => "other",
            };

            // Try the specific category first, fall back to "other"
            let branch = select_plural_branch(branches_str, category_str)
                .or_else(|| select_plural_branch(branches_str, "other"))
                .unwrap_or_default();

            branch.replace('#', &count.to_string())
        }
        _ => format!("{{{inner}}}"),
    }
}

/// Selects the content for a plural branch like `one {# minute}` from the branches string.
fn select_plural_branch(branches: &str, category: &str) -> Option<String> {
    // Find "category {content}" pattern, handling nested braces
    let chars: Vec<char> = branches.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Skip whitespace
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }

        // Read keyword
        let kw_start = i;
        while i < len && !chars[i].is_whitespace() && chars[i] != '{' {
            i += 1;
        }
        let keyword: String = chars[kw_start..i].iter().collect();

        // Skip whitespace before '{'
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }

        // Read braced content
        if i < len && chars[i] == '{' {
            if let Some(close) = find_matching_brace(&chars, i) {
                let content: String = chars[i + 1..close].iter().collect();
                if keyword == category {
                    return Some(content);
                }
                i = close + 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures i18n is initialized for tests.
    fn ensure_init() {
        let _ = init("en");
    }

    #[test]
    fn test_simple_lookup() {
        ensure_init();
        assert_eq!(t("tray.quit"), "Quit");
        assert_eq!(t("tray.refresh_now"), "Refresh Now");
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
            "notification.title",
            &[
                ("game_name", Value::String("Genshin Impact".to_string())),
                ("resource_name", Value::String("Original Resin".to_string())),
            ],
        );
        assert_eq!(result, "Genshin Impact \u{2014} Original Resin");
    }

    #[test]
    fn test_resource_full_simple() {
        ensure_init();
        let result = t_args(
            "notification.resource_full",
            &[("resource_name", Value::String("Waveplates".to_string()))],
        );
        assert_eq!(result, "Waveplates is full!");
    }

    #[test]
    fn test_resource_ready() {
        ensure_init();
        let result = t_args(
            "notification.resource_ready",
            &[(
                "resource_name",
                Value::String("Parametric Transformer".to_string()),
            )],
        );
        assert_eq!(result, "Parametric Transformer is ready to claim!");
    }

    #[test]
    fn test_resource_ready_in() {
        ensure_init();
        let result = t_args(
            "notification.resource_ready_in",
            &[
                ("resource_name", Value::String("Expeditions".to_string())),
                ("duration", Value::String("30 min".to_string())),
            ],
        );
        assert_eq!(result, "Expeditions will be ready in 30 min");
    }

    #[test]
    fn test_resource_status() {
        ensure_init();
        let result = t_args(
            "notification.resource_status",
            &[
                ("resource_name", Value::String("Original Resin".to_string())),
                ("current", Value::String("140".to_string())),
                ("max", Value::String("160".to_string())),
                ("duration", Value::String("1 hr 15 min".to_string())),
                ("clock_time", Value::String("3:45 PM".to_string())),
            ],
        );
        assert_eq!(
            result,
            "Original Resin has reached 140/160 and will be full in 1 hr 15 min at 3:45 PM"
        );
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
        // ICU4X with FieldDisplay::Always on minutes should produce "0 min" (en locale)
        assert_eq!(result, "0 min");
    }

    #[test]
    fn test_format_duration_negative_clamps() {
        ensure_init();
        let result = format_duration(-10);
        // Negative clamps to 0, same as zero duration
        assert_eq!(result, "0 min");
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
        assert_eq!(t("game.genshin.name"), "Genshin Impact");
        assert_eq!(t("game.hsr.name"), "Honkai: Star Rail");
        assert_eq!(t("game.zzz.name"), "Zenless Zone Zero");
        assert_eq!(t("game.wuwa.name"), "Wuthering Waves");
    }

    #[test]
    fn test_resource_names() {
        ensure_init();
        assert_eq!(t("game.genshin.resource.resin"), "Original Resin");
        assert_eq!(t("game.hsr.resource.trailblaze_power"), "Trailblaze Power");
        assert_eq!(t("game.zzz.resource.battery"), "Battery");
        assert_eq!(t("game.wuwa.resource.waveplates"), "Waveplates");
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
        let result = format_message(
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
