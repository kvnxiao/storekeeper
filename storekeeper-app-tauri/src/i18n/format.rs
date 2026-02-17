use icu_datetime::fieldsets;
use icu_experimental::duration::{
    DurationFormatter, DurationFormatterPreferences, ValidatedDurationFormatterOptions,
    options::{BaseStyle, DurationFormatterOptions, FieldDisplay},
};

use super::store::with_messages;

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
/// Uses `icu_experimental::duration::DurationFormatter` with `BaseStyle::Narrow`
/// (e.g. "1h 15m" in English). Clamps negative values to 0.
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
        opts.base = BaseStyle::Narrow;
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
