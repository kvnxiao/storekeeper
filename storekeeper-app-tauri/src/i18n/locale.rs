/// List of supported locale codes.
pub(super) const SUPPORTED_LOCALES: &[&str] = &["en", "zh-CN", "ko", "ja"];

/// Default locale used when no match is found.
pub(super) const DEFAULT_LOCALE: &str = "en";

/// Matches a locale string against supported locales.
///
/// Tries exact match first, then language-prefix match (e.g. "en-US" -> "en").
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
