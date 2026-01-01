//! Common serde utilities for deserialization.

/// Deserializes a string containing seconds to `DateTime<Local>`.
///
/// Adds the seconds to the current local time to produce a future DateTime.
/// If the value is "0" or empty, returns the current local time.
///
/// # Errors
///
/// Returns an error if the value is negative or cannot be parsed.
pub mod seconds_string_to_datetime {
    use chrono::{DateTime, Local, TimeDelta};
    use serde::{Deserialize, Deserializer};

    /// Deserializes a string of seconds into `DateTime<Local>`.
    ///
    /// Returns `Local::now()` if the value is zero.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error if the seconds value is negative or invalid.
    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let secs: i64 = s
            .parse()
            .map_err(|_| serde::de::Error::custom("invalid seconds string"))?;
        if secs < 0 {
            return Err(serde::de::Error::custom("seconds must not be negative"));
        }
        if secs == 0 {
            return Ok(Local::now());
        }
        let delta = TimeDelta::try_seconds(secs)
            .ok_or_else(|| serde::de::Error::custom("invalid seconds value"))?;
        Ok(Local::now() + delta)
    }
}

/// Deserializes a `u64` containing seconds to `DateTime<Local>`.
///
/// Adds the seconds to the current local time to produce a future DateTime.
/// If the value is 0, returns the current local time.
///
/// # Errors
///
/// Returns an error if conversion fails.
pub mod seconds_u64_to_datetime {
    use chrono::{DateTime, Local, TimeDelta};
    use serde::{Deserialize, Deserializer};

    /// Deserializes a u64 of seconds into `DateTime<Local>`.
    ///
    /// Returns `Local::now()` if the value is zero.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error if the seconds value is invalid.
    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        if secs == 0 {
            return Ok(Local::now());
        }
        let secs_i64 =
            i64::try_from(secs).map_err(|_| serde::de::Error::custom("seconds value too large"))?;
        let delta = TimeDelta::try_seconds(secs_i64)
            .ok_or_else(|| serde::de::Error::custom("invalid seconds value"))?;
        Ok(Local::now() + delta)
    }
}

/// Deserializes a `u64` millisecond timestamp to `DateTime<Local>`.
///
/// Converts the absolute timestamp to a local DateTime.
/// If the value is 0, returns the current local time.
///
/// # Errors
///
/// Returns an error if conversion fails.
pub mod timestamp_ms_to_datetime {
    use chrono::{DateTime, Local, TimeZone, Utc};
    use serde::{Deserialize, Deserializer};

    /// Deserializes a u64 millisecond timestamp into `DateTime<Local>`.
    ///
    /// Returns `Local::now()` if the value is zero.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error if the timestamp is invalid.
    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp_ms = u64::deserialize(deserializer)?;
        if timestamp_ms == 0 {
            return Ok(Local::now());
        }
        let timestamp_ms_i64 = i64::try_from(timestamp_ms)
            .map_err(|_| serde::de::Error::custom("timestamp value too large"))?;

        Utc.timestamp_millis_opt(timestamp_ms_i64)
            .single()
            .map(|dt| dt.with_timezone(&Local))
            .ok_or_else(|| serde::de::Error::custom("invalid timestamp"))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use serde::Deserialize;

    // =========================================================================
    // seconds_string_to_datetime tests
    // =========================================================================

    #[derive(Debug, Deserialize)]
    struct TestSecondsString {
        #[serde(deserialize_with = "super::seconds_string_to_datetime::deserialize")]
        value: chrono::DateTime<Local>,
    }

    #[test]
    fn test_seconds_string_zero_returns_now() {
        let json = r#"{"value": "0"}"#;
        let before = Local::now();
        let result: TestSecondsString =
            serde_json::from_str(json).expect("should deserialize zero");
        let after = Local::now();

        // The result should be between before and after (i.e., approximately now)
        assert!(
            result.value >= before && result.value <= after,
            "Zero seconds should return approximately Local::now()"
        );
    }

    #[test]
    fn test_seconds_string_positive_adds_to_now() {
        let json = r#"{"value": "60"}"#;
        let before = Local::now();
        let result: TestSecondsString =
            serde_json::from_str(json).expect("should deserialize 60 seconds");
        let after = Local::now();

        // Result should be approximately 60 seconds in the future
        let diff_from_before = (result.value - before).num_seconds();
        let diff_from_after = (result.value - after).num_seconds();

        assert!(
            (59..=61).contains(&diff_from_before),
            "60 seconds should add ~60s to now, got diff: {diff_from_before}"
        );
        assert!(
            (59..=61).contains(&diff_from_after),
            "60 seconds should add ~60s to now, got diff: {diff_from_after}"
        );
    }

    #[test]
    fn test_seconds_string_negative_fails() {
        let json = r#"{"value": "-1"}"#;
        let result: Result<TestSecondsString, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Negative seconds should fail deserialization"
        );
    }

    #[test]
    fn test_seconds_string_invalid_fails() {
        let json = r#"{"value": "not_a_number"}"#;
        let result: Result<TestSecondsString, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Invalid string should fail deserialization"
        );
    }

    // =========================================================================
    // seconds_u64_to_datetime tests
    // =========================================================================

    #[derive(Debug, Deserialize)]
    struct TestSecondsU64 {
        #[serde(deserialize_with = "super::seconds_u64_to_datetime::deserialize")]
        value: chrono::DateTime<Local>,
    }

    #[test]
    fn test_seconds_u64_zero_returns_now() {
        let json = r#"{"value": 0}"#;
        let before = Local::now();
        let result: TestSecondsU64 = serde_json::from_str(json).expect("should deserialize zero");
        let after = Local::now();

        assert!(
            result.value >= before && result.value <= after,
            "Zero seconds should return approximately Local::now()"
        );
    }

    #[test]
    fn test_seconds_u64_positive_adds_to_now() {
        let json = r#"{"value": 120}"#;
        let before = Local::now();
        let result: TestSecondsU64 =
            serde_json::from_str(json).expect("should deserialize 120 seconds");

        let diff = (result.value - before).num_seconds();
        assert!(
            (119..=121).contains(&diff),
            "120 seconds should add ~120s to now, got diff: {diff}"
        );
    }

    // =========================================================================
    // timestamp_ms_to_datetime tests
    // =========================================================================

    #[derive(Debug, Deserialize)]
    struct TestTimestampMs {
        #[serde(deserialize_with = "super::timestamp_ms_to_datetime::deserialize")]
        value: chrono::DateTime<Local>,
    }

    #[test]
    fn test_timestamp_ms_zero_returns_now() {
        let json = r#"{"value": 0}"#;
        let before = Local::now();
        let result: TestTimestampMs = serde_json::from_str(json).expect("should deserialize zero");
        let after = Local::now();

        assert!(
            result.value >= before && result.value <= after,
            "Zero timestamp should return approximately Local::now()"
        );
    }

    #[test]
    fn test_timestamp_ms_valid_converts_correctly() {
        // Use a known timestamp: 2024-01-01 00:00:00 UTC = 1704067200000 ms
        let json = r#"{"value": 1704067200000}"#;
        let result: TestTimestampMs =
            serde_json::from_str(json).expect("should deserialize valid timestamp");

        // The UTC time should be 2024-01-01 00:00:00
        let utc = result.value.naive_utc();
        assert_eq!(utc.and_utc().timestamp(), 1_704_067_200);
    }

    #[test]
    fn test_timestamp_ms_future_works() {
        // Use a future timestamp (current time + 1 hour in ms)
        let future_ms = chrono::Utc::now().timestamp_millis() + 3_600_000;
        let json = format!(r#"{{"value": {future_ms}}}"#);
        let result: TestTimestampMs =
            serde_json::from_str(&json).expect("should deserialize future timestamp");

        // Should be approximately 1 hour in the future
        let diff = (result.value - Local::now()).num_seconds();
        assert!(
            (3590..=3610).contains(&diff),
            "Future timestamp should be ~1 hour from now, got diff: {diff}s"
        );
    }
}
