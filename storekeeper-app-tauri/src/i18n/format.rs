use chrono::{DateTime, Datelike, Local, Timelike};
use icu_datetime::fieldsets;
use icu_experimental::duration::{
    DurationFormatter, DurationFormatterPreferences, ValidatedDurationFormatterOptions,
    options::{BaseStyle, DurationFormatterOptions, FieldDisplay},
};

use super::store::with_messages;

/// Formats a completion time using the current locale.
///
/// When the completion is on the same calendar day as `now`, shows time only
/// (e.g. "3:45 PM"). When on a different day, shows weekday + time
/// (e.g. "Mon 3:45 PM" / "月 15:45") using ICU4X locale-aware formatting.
#[must_use]
pub fn format_time(completion: DateTime<Local>, now: DateTime<Local>) -> String {
    let is_today = completion.date_naive() == now.date_naive();

    if is_today {
        format_time_only(completion)
    } else {
        format_weekday_time(completion)
    }
}

/// Formats just the time portion (hour + minute) using the current locale.
fn format_time_only(dt: DateTime<Local>) -> String {
    let hour = u8::try_from(dt.hour()).unwrap_or(0);
    let minute = u8::try_from(dt.minute()).unwrap_or(0);
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

/// Formats weekday + time (e.g. "Mon 3:45 PM") using ICU4X locale-aware formatting.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn format_weekday_time(dt: DateTime<Local>) -> String {
    let hour = u8::try_from(dt.hour()).unwrap_or(0);
    let minute = u8::try_from(dt.minute()).unwrap_or(0);
    let fallback = || {
        let weekday = dt.format("%a").to_string();
        format!("{weekday} {hour}:{minute:02}")
    };

    with_messages(|m| {
        let year = dt.year();
        let month = dt.month() as u8;
        let day = dt.day() as u8;
        let Ok(date) = icu_calendar::Date::try_new_iso(year, month, day) else {
            return fallback();
        };
        let Ok(time) = icu_time::Time::try_new(hour, minute, 0, 0) else {
            return fallback();
        };
        let input = icu_datetime::input::DateTime { date, time };
        let Ok(formatter) = icu_datetime::DateTimeFormatter::try_new(
            m.locale.clone().into(),
            fieldsets::ET::short(),
        ) else {
            return fallback();
        };
        formatter.format(&input).to_string()
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
    let days = clamped / 1440;
    let hours = (clamped % 1440) / 60;
    let minutes = clamped % 60;

    let fallback = || {
        if days > 0 {
            format!("{days}d {hours}h {minutes}m")
        } else if hours > 0 {
            format!("{hours}h {minutes}m")
        } else {
            format!("{minutes}m")
        }
    };

    with_messages(|m| {
        let mut opts = DurationFormatterOptions::default();
        // Narrow style only works correctly for English; use Short for all other locales.
        opts.base = if m.locale.id.language == icu_locale::subtags::language!("en") {
            BaseStyle::Narrow
        } else {
            BaseStyle::Short
        };
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
            days,
            hours,
            minutes,
            ..Default::default()
        };
        formatter.format(&duration).to_string()
    })
    .unwrap_or_else(fallback)
}
