use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use anyhow::{Context, Result, bail};
use icu_locale::Locale;
use icu_plurals::PluralRules;

use super::locale::{DEFAULT_LOCALE, SUPPORTED_LOCALES};

/// Embedded English locale strings (loaded at compile time).
const EN_LOCALE: &str = include_str!("../../../locales/en.json");

/// Global messages store, initialized once at startup and switchable at runtime.
static MESSAGES: OnceLock<RwLock<Messages>> = OnceLock::new();

/// Holds the loaded locale data: parsed strings and locale info.
///
/// `PluralRules` is not stored here because it is `!Send + !Sync` (uses `Rc`
/// internally). Instead, plural rules are created on-demand in `t_args`.
pub(super) struct Messages {
    pub(super) locale: Locale,
    pub(super) strings: HashMap<String, String>,
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

/// Acquires a read lock on the global messages and runs the closure.
pub(super) fn with_messages<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&Messages) -> R,
{
    let lock = MESSAGES.get()?;
    let guard = lock.read().ok()?;
    Some(f(&guard))
}
