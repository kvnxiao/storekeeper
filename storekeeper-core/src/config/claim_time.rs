//! [`ClaimTime`] type and UTC+8 serde helpers for daily reward claiming.

use chrono::NaiveTime;

use crate::error::{Error, Result};

/// UTC+8 offset in seconds (8 hours = 28800 seconds).
const UTC8_OFFSET_SECS: i32 = 8 * 3600;

/// Default claim time in UTC+8 (midnight), displayed as "00:00".
pub const DEFAULT_AUTO_CLAIM_TIME: &str = "00:00";

/// A time of day stored in UTC for daily reward claiming.
///
/// Internally stores time in UTC. When serialized to config files,
/// the time is displayed as UTC+8 (China Standard Time) in "HH:MM" format.
///
/// # Examples
///
/// ```
/// use storekeeper_core::ClaimTime;
///
/// // Parse from UTC+8 string (08:30 UTC+8 = 00:30 UTC)
/// let time = ClaimTime::from_utc8_str("08:30").unwrap();
/// assert_eq!(time.as_naive_time().to_string(), "00:30:00");
///
/// // Convert back to UTC+8 string
/// assert_eq!(time.to_utc8_string(), "08:30");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimTime(NaiveTime);

impl ClaimTime {
    /// Creates a `ClaimTime` from a UTC+8 time string in strict HH:MM format.
    ///
    /// The time is converted to UTC for internal storage.
    ///
    /// # Arguments
    ///
    /// * `time_str` - A time string in HH:MM format representing UTC+8 time.
    ///
    /// # Errors
    ///
    /// Returns an error if the time string is not in valid HH:MM format.
    ///
    /// # Examples
    ///
    /// ```
    /// use storekeeper_core::ClaimTime;
    ///
    /// // 08:30 UTC+8 = 00:30 UTC
    /// let time = ClaimTime::from_utc8_str("08:30").unwrap();
    ///
    /// // Invalid formats return errors
    /// assert!(ClaimTime::from_utc8_str("8:30").is_err());
    /// assert!(ClaimTime::from_utc8_str("25:00").is_err());
    /// ```
    pub fn from_utc8_str(time_str: &str) -> Result<Self> {
        // Strict validation: must be exactly 5 characters (HH:MM)
        if time_str.len() != 5 {
            return Err(Error::ConfigParseFailed {
                message: format!(
                    "Invalid claim_time format: '{time_str}'. Expected HH:MM (e.g., '00:10')"
                ),
            });
        }

        // Must have colon at position 2
        if time_str.chars().nth(2) != Some(':') {
            return Err(Error::ConfigParseFailed {
                message: format!(
                    "Invalid claim_time format: '{time_str}'. Expected HH:MM with colon separator"
                ),
            });
        }

        // Parse using chrono
        let parsed_utc8_time =
            NaiveTime::parse_from_str(time_str, "%H:%M").map_err(|e| Error::ConfigParseFailed {
                message: format!("Invalid claim_time '{time_str}': {e}"),
            })?;

        // Convert from UTC+8 to UTC (subtract 8 hours)
        // NaiveTime arithmetic handles the wrap-around correctly
        let converted_utc_time =
            parsed_utc8_time - chrono::Duration::seconds(i64::from(UTC8_OFFSET_SECS));

        Ok(Self(converted_utc_time))
    }

    /// Returns midnight in UTC+8 (00:00 UTC+8 = 16:00 UTC previous day).
    ///
    /// This is the default claim time when none is specified.
    #[must_use = "this returns the default claim time, it doesn't modify anything"]
    pub fn default_utc8_midnight() -> Self {
        // 00:00 UTC+8 = 16:00 UTC (previous day)
        // SAFETY: 16:00:00 is always a valid time, so unwrap_or provides
        // a fallback that will never be used in practice.
        Self(
            NaiveTime::from_hms_opt(16, 0, 0)
                .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap_or(NaiveTime::MIN)),
        )
    }

    /// Returns the inner `NaiveTime` (in UTC).
    #[must_use = "this returns the time value, it doesn't modify anything"]
    pub fn as_naive_time(&self) -> NaiveTime {
        self.0
    }

    /// Returns this time formatted as a UTC+8 "HH:MM" string.
    ///
    /// This is the inverse of `from_utc8_str`.
    #[must_use = "this returns the formatted string, it doesn't modify anything"]
    pub fn to_utc8_string(&self) -> String {
        // Convert from UTC to UTC+8 (add 8 hours)
        let utc8_time = self.0 + chrono::Duration::seconds(i64::from(UTC8_OFFSET_SECS));
        utc8_time.format("%H:%M").to_string()
    }
}

impl std::fmt::Display for ClaimTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Display as UTC+8 for human readability
        write!(f, "{}", self.to_utc8_string())
    }
}

/// Calculates the next claim datetime in UTC for a given claim time.
///
/// Takes a claim time (already stored in UTC internally), and determines
/// whether the next occurrence is today or tomorrow.
///
/// # Arguments
///
/// * `claim_time` - Optional claim time. Defaults to midnight UTC+8 if None.
///
/// # Errors
///
/// Returns an error if the datetime calculation fails.
pub fn next_claim_datetime_utc(
    claim_time: Option<ClaimTime>,
) -> Result<chrono::DateTime<chrono::Utc>> {
    use chrono::{Datelike, TimeZone, Timelike, Utc};

    // Use provided time or default to midnight UTC+8 (which is 16:00 UTC)
    let time = claim_time.unwrap_or_else(ClaimTime::default_utc8_midnight);
    let utc_time = time.as_naive_time();

    // Current time in UTC
    let now = Utc::now();

    // Today's claim time in UTC
    let today_claim_utc = Utc
        .with_ymd_and_hms(
            now.year(),
            now.month(),
            now.day(),
            utc_time.hour(),
            utc_time.minute(),
            0,
        )
        .single()
        .ok_or_else(|| Error::ConfigParseFailed {
            message: "Failed to construct claim datetime".to_string(),
        })?;

    // If today's claim time has passed, use tomorrow
    let next_claim = if now >= today_claim_utc {
        today_claim_utc + chrono::Duration::days(1)
    } else {
        today_claim_utc
    };

    Ok(next_claim)
}

/// Serde module for serializing/deserializing `Option<ClaimTime>`.
///
/// - **Deserialize:** Parses "HH:MM" as UTC+8, converts to UTC for storage.
/// - **Serialize:** Converts UTC to UTC+8, formats as "HH:MM".
pub(crate) mod claim_time_serde {
    use super::ClaimTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    #[allow(clippy::ref_option)] // serde's `with` requires &Option<T> signature
    pub fn serialize<S>(time: &Option<ClaimTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match time {
            Some(t) => serializer.serialize_some(&t.to_utc8_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ClaimTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => ClaimTime::from_utc8_str(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    use serde::{Deserialize, Serialize};

    // =========================================================================
    // ClaimTime::from_utc8_str tests
    // =========================================================================

    #[test]
    fn test_claim_time_valid_formats() {
        // Standard cases - all should parse successfully
        assert!(
            ClaimTime::from_utc8_str("00:00").is_ok(),
            "00:00 should be valid (midnight UTC+8)"
        );
        assert!(
            ClaimTime::from_utc8_str("00:10").is_ok(),
            "00:10 should be valid"
        );
        assert!(
            ClaimTime::from_utc8_str("12:30").is_ok(),
            "12:30 should be valid (noon UTC+8)"
        );
        assert!(
            ClaimTime::from_utc8_str("23:59").is_ok(),
            "23:59 should be valid (end of day UTC+8)"
        );
        assert!(
            ClaimTime::from_utc8_str("08:00").is_ok(),
            "08:00 should be valid (morning UTC+8)"
        );
        assert!(
            ClaimTime::from_utc8_str("16:45").is_ok(),
            "16:45 should be valid (afternoon UTC+8)"
        );
    }

    #[test]
    fn test_claim_time_utc_conversion() {
        // 08:30 UTC+8 = 00:30 UTC (subtract 8 hours)
        let time = ClaimTime::from_utc8_str("08:30").expect("08:30 should be valid");
        assert_eq!(time.as_naive_time().hour(), 0);
        assert_eq!(time.as_naive_time().minute(), 30);

        // 16:00 UTC+8 = 08:00 UTC
        let time = ClaimTime::from_utc8_str("16:00").expect("16:00 should be valid");
        assert_eq!(time.as_naive_time().hour(), 8);
        assert_eq!(time.as_naive_time().minute(), 0);

        // 23:59 UTC+8 = 15:59 UTC
        let time = ClaimTime::from_utc8_str("23:59").expect("23:59 should be valid");
        assert_eq!(time.as_naive_time().hour(), 15);
        assert_eq!(time.as_naive_time().minute(), 59);
    }

    #[test]
    fn test_claim_time_midnight_utc8_conversion() {
        // 00:00 UTC+8 = 16:00 UTC (previous day, wraps around)
        let time = ClaimTime::from_utc8_str("00:00").expect("00:00 should be valid");
        assert_eq!(time.as_naive_time().hour(), 16);
        assert_eq!(time.as_naive_time().minute(), 0);

        // 07:59 UTC+8 = 23:59 UTC (previous day)
        let time = ClaimTime::from_utc8_str("07:59").expect("07:59 should be valid");
        assert_eq!(time.as_naive_time().hour(), 23);
        assert_eq!(time.as_naive_time().minute(), 59);
    }

    #[test]
    fn test_claim_time_to_utc8_string_roundtrip() {
        // Test that to_utc8_string() is the inverse of from_utc8_str()
        let test_times = ["00:00", "00:10", "08:30", "12:00", "16:45", "23:59"];

        for time_str in test_times {
            let time = ClaimTime::from_utc8_str(time_str).expect("all test times should be valid");
            assert_eq!(
                time.to_utc8_string(),
                time_str,
                "Round-trip failed for {time_str}"
            );
        }
    }

    #[test]
    fn test_claim_time_display() {
        // Display should show UTC+8 time for human readability
        let time = ClaimTime::from_utc8_str("08:30").expect("08:30 should be valid");
        assert_eq!(format!("{time}"), "08:30");
    }

    #[test]
    fn test_claim_time_default_utc8_midnight() {
        let default = ClaimTime::default_utc8_midnight();
        // 00:00 UTC+8 = 16:00 UTC
        assert_eq!(default.as_naive_time().hour(), 16);
        assert_eq!(default.as_naive_time().minute(), 0);
        // Should display as midnight UTC+8
        assert_eq!(default.to_utc8_string(), "00:00");
    }

    #[test]
    fn test_claim_time_invalid_empty() {
        assert!(
            ClaimTime::from_utc8_str("").is_err(),
            "Empty string should be invalid"
        );
    }

    #[test]
    fn test_claim_time_invalid_missing_leading_zero() {
        assert!(
            ClaimTime::from_utc8_str("0:00").is_err(),
            "0:00 should be invalid (missing leading zero in hour)"
        );
        assert!(
            ClaimTime::from_utc8_str("00:0").is_err(),
            "00:0 should be invalid (missing leading zero in minute)"
        );
        assert!(
            ClaimTime::from_utc8_str("9:30").is_err(),
            "9:30 should be invalid (missing leading zero)"
        );
    }

    #[test]
    fn test_claim_time_invalid_out_of_range() {
        assert!(
            ClaimTime::from_utc8_str("24:00").is_err(),
            "24:00 should be invalid (hour out of range)"
        );
        assert!(
            ClaimTime::from_utc8_str("00:60").is_err(),
            "00:60 should be invalid (minute out of range)"
        );
        assert!(
            ClaimTime::from_utc8_str("25:30").is_err(),
            "25:30 should be invalid (hour out of range)"
        );
        assert!(
            ClaimTime::from_utc8_str("12:99").is_err(),
            "12:99 should be invalid (minute out of range)"
        );
    }

    #[test]
    fn test_claim_time_invalid_format() {
        assert!(
            ClaimTime::from_utc8_str("12:30:00").is_err(),
            "12:30:00 should be invalid (includes seconds)"
        );
        assert!(
            ClaimTime::from_utc8_str("12-30").is_err(),
            "12-30 should be invalid (wrong separator)"
        );
        assert!(
            ClaimTime::from_utc8_str("1230").is_err(),
            "1230 should be invalid (no separator)"
        );
    }

    #[test]
    fn test_claim_time_invalid_non_numeric() {
        assert!(
            ClaimTime::from_utc8_str("abc").is_err(),
            "abc should be invalid (non-numeric)"
        );
        assert!(
            ClaimTime::from_utc8_str("12:ab").is_err(),
            "12:ab should be invalid (partial non-numeric)"
        );
        assert!(
            ClaimTime::from_utc8_str("ab:30").is_err(),
            "ab:30 should be invalid (partial non-numeric)"
        );
        assert!(
            ClaimTime::from_utc8_str("HH:MM").is_err(),
            "HH:MM should be invalid (placeholder text)"
        );
    }

    #[test]
    fn test_claim_time_invalid_whitespace() {
        assert!(
            ClaimTime::from_utc8_str(" 00:10").is_err(),
            " 00:10 should be invalid (leading whitespace)"
        );
        assert!(
            ClaimTime::from_utc8_str("00:10 ").is_err(),
            "00:10  should be invalid (trailing whitespace)"
        );
        assert!(
            ClaimTime::from_utc8_str("00 :10").is_err(),
            "00 :10 should be invalid (internal whitespace)"
        );
    }

    // =========================================================================
    // Serde tests
    // =========================================================================

    #[test]
    fn test_claim_time_serde_roundtrip() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestConfig {
            #[serde(default, with = "claim_time_serde")]
            time: Option<ClaimTime>,
        }

        // Test with value
        let toml_str = r#"time = "08:30""#;
        let config: TestConfig = toml::from_str(toml_str).expect("should deserialize");
        let time = config.time.expect("should have time");
        assert_eq!(time.to_utc8_string(), "08:30");

        // Serialize back
        let serialized = toml::to_string(&config).expect("should serialize");
        assert!(
            serialized.contains("time = \"08:30\""),
            "serialized should contain time = \"08:30\", got: {serialized}"
        );

        // Test with None (missing field)
        let toml_str = "";
        let config: TestConfig = toml::from_str(toml_str).expect("should deserialize empty");
        assert!(
            config.time.is_none(),
            "time should be None for empty config"
        );
    }

    #[test]
    fn test_claim_time_serde_invalid_format() {
        #[derive(Debug, Serialize, Deserialize)]
        struct TestConfig {
            #[serde(default, with = "claim_time_serde")]
            time: Option<ClaimTime>,
        }

        // Invalid time format should fail
        let toml_str = r#"time = "invalid""#;
        let result: std::result::Result<TestConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err(), "invalid time format should fail to parse");
    }

    // =========================================================================
    // next_claim_datetime_utc tests
    // =========================================================================

    #[test]
    fn test_default_auto_claim_time_constant() {
        assert_eq!(DEFAULT_AUTO_CLAIM_TIME, "00:00");
    }

    #[test]
    fn test_next_claim_datetime_utc_with_default() {
        // Should successfully calculate next claim time with default (None)
        let result = next_claim_datetime_utc(None);
        assert!(
            result.is_ok(),
            "next_claim_datetime_utc should succeed with None (default)"
        );

        // The result should be in the future or within seconds of now
        let next_claim = result.expect("should be valid");
        let now = chrono::Utc::now();

        // The next claim should be within the next 24 hours + a few seconds of tolerance
        let diff = next_claim - now;
        assert!(
            diff.num_seconds() >= -5,
            "Next claim time should be in the future (or very recent)"
        );
        assert!(
            diff.num_hours() <= 24,
            "Next claim time should be within 24 hours"
        );
    }

    #[test]
    fn test_next_claim_datetime_utc_with_custom_time() {
        // Should successfully calculate next claim time with custom time
        let claim_time =
            ClaimTime::from_utc8_str("08:30").expect("08:30 should be valid UTC+8 time");
        let result = next_claim_datetime_utc(Some(claim_time));
        assert!(
            result.is_ok(),
            "next_claim_datetime_utc should succeed with valid time"
        );

        let next_claim = result.expect("should be valid");
        let now = chrono::Utc::now();

        let diff = next_claim - now;
        assert!(
            diff.num_seconds() >= -5,
            "Next claim time should be in the future (or very recent)"
        );
        assert!(
            diff.num_hours() <= 24,
            "Next claim time should be within 24 hours"
        );
    }

    #[test]
    fn test_claim_time_copy_semantics() {
        // ClaimTime should be Copy, so we can use it without cloning
        let time = ClaimTime::from_utc8_str("08:30").expect("08:30 should be valid");
        let time2 = time; // Copy
        let time3 = time; // Copy again

        assert_eq!(time.to_utc8_string(), time2.to_utc8_string());
        assert_eq!(time.to_utc8_string(), time3.to_utc8_string());
    }
}
