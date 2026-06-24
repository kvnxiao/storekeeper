//! Common serde utilities for deserializing API timestamp fields.
//!
//! HoYoLab and Kuro APIs encode resource completion times in three shapes:
//! a seconds-from-now string, a seconds-from-now `u64`, and an absolute
//! millisecond `u64` timestamp. Each shape gets its own thin public module
//! (serde's `with =`/`deserialize_with =` requires a module path) that does the
//! shape-specific parsing and delegates the conversion to the shared helpers
//! below.

use jiff::{SignedDuration, Timestamp};

/// Converts a non-negative count of seconds-from-now into an absolute instant.
///
/// Returns the current instant when `secs` is zero. Overflow past the
/// representable [`Timestamp`] range is reported as an error rather than
/// panicking, since `secs` originates from an external API response.
fn timestamp_from_secs_from_now(secs: i64) -> Result<Timestamp, &'static str> {
    if secs == 0 {
        return Ok(Timestamp::now());
    }
    Timestamp::now()
        .checked_add(SignedDuration::from_secs(secs))
        .map_err(|_| "seconds value out of representable range")
}

/// Converts an absolute Unix millisecond timestamp into an instant.
///
/// Returns the current instant when `millis` is zero.
fn timestamp_from_millis(millis: i64) -> Result<Timestamp, &'static str> {
    if millis == 0 {
        return Ok(Timestamp::now());
    }
    Timestamp::from_millisecond(millis).map_err(|_| "invalid millisecond timestamp")
}

/// Deserializes a string containing seconds-from-now into a [`Timestamp`].
///
/// Adds the seconds to the current instant to produce a future timestamp.
/// If the value is `"0"`, returns the current instant.
pub mod seconds_string_to_datetime {
    use jiff::Timestamp;
    use serde::{Deserialize, Deserializer};

    /// Deserializes a string of seconds-from-now into a [`Timestamp`].
    ///
    /// Returns `Timestamp::now()` if the value is zero.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error if the value is not an integer, is
    /// negative, or would overflow the representable timestamp range.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
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
        super::timestamp_from_secs_from_now(secs).map_err(serde::de::Error::custom)
    }
}

/// Deserializes a `u64` containing seconds-from-now into a [`Timestamp`].
///
/// Adds the seconds to the current instant to produce a future timestamp.
/// If the value is 0, returns the current instant.
pub mod seconds_u64_to_datetime {
    use jiff::Timestamp;
    use serde::{Deserialize, Deserializer};

    /// Deserializes a `u64` of seconds-from-now into a [`Timestamp`].
    ///
    /// Returns `Timestamp::now()` if the value is zero.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error if the seconds value exceeds [`i64::MAX`]
    /// or would overflow the representable timestamp range.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        let secs =
            i64::try_from(secs).map_err(|_| serde::de::Error::custom("seconds value too large"))?;
        super::timestamp_from_secs_from_now(secs).map_err(serde::de::Error::custom)
    }
}

/// Deserializes a `u64` absolute millisecond timestamp into a [`Timestamp`].
///
/// Converts the absolute Unix-millisecond value to an instant.
/// If the value is 0, returns the current instant.
pub mod timestamp_ms_to_datetime {
    use jiff::Timestamp;
    use serde::{Deserialize, Deserializer};

    /// Deserializes a `u64` millisecond timestamp into a [`Timestamp`].
    ///
    /// Returns `Timestamp::now()` if the value is zero.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error if the value exceeds [`i64::MAX`] or is
    /// outside the representable timestamp range.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        let millis = i64::try_from(millis)
            .map_err(|_| serde::de::Error::custom("timestamp value too large"))?;
        super::timestamp_from_millis(millis).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;
    use serde::Deserialize;

    // =========================================================================
    // seconds_string_to_datetime tests
    // =========================================================================

    #[derive(Debug, Deserialize)]
    struct TestSecondsString {
        #[serde(deserialize_with = "super::seconds_string_to_datetime::deserialize")]
        value: Timestamp,
    }

    #[test]
    fn test_seconds_string_zero_returns_now() {
        let json = r#"{"value": "0"}"#;
        let before = Timestamp::now();
        let result: TestSecondsString =
            serde_json::from_str(json).expect("should deserialize zero");
        let after = Timestamp::now();

        // The result should be between before and after (i.e., approximately now)
        assert!(
            result.value >= before && result.value <= after,
            "Zero seconds should return approximately Timestamp::now()"
        );
    }

    #[test]
    fn test_seconds_string_positive_adds_to_now() {
        let json = r#"{"value": "60"}"#;
        let before = Timestamp::now();
        let result: TestSecondsString =
            serde_json::from_str(json).expect("should deserialize 60 seconds");
        let after = Timestamp::now();

        // Result should be approximately 60 seconds in the future
        let diff_from_before = result.value.duration_since(before).as_secs();
        let diff_from_after = result.value.duration_since(after).as_secs();

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
        value: Timestamp,
    }

    #[test]
    fn test_seconds_u64_zero_returns_now() {
        let json = r#"{"value": 0}"#;
        let before = Timestamp::now();
        let result: TestSecondsU64 = serde_json::from_str(json).expect("should deserialize zero");
        let after = Timestamp::now();

        assert!(
            result.value >= before && result.value <= after,
            "Zero seconds should return approximately Timestamp::now()"
        );
    }

    #[test]
    fn test_seconds_u64_positive_adds_to_now() {
        let json = r#"{"value": 120}"#;
        let before = Timestamp::now();
        let result: TestSecondsU64 =
            serde_json::from_str(json).expect("should deserialize 120 seconds");

        let diff = result.value.duration_since(before).as_secs();
        assert!(
            (119..=121).contains(&diff),
            "120 seconds should add ~120s to now, got diff: {diff}"
        );
    }

    #[test]
    fn test_seconds_u64_overflow_fails() {
        // u64::MAX exceeds i64::MAX, so the conversion must reject it.
        let json = format!(r#"{{"value": {}}}"#, u64::MAX);
        let result: Result<TestSecondsU64, _> = serde_json::from_str(&json);
        assert!(
            result.is_err(),
            "u64::MAX seconds should fail deserialization"
        );
    }

    // =========================================================================
    // timestamp_ms_to_datetime tests
    // =========================================================================

    #[derive(Debug, Deserialize)]
    struct TestTimestampMs {
        #[serde(deserialize_with = "super::timestamp_ms_to_datetime::deserialize")]
        value: Timestamp,
    }

    #[test]
    fn test_timestamp_ms_zero_returns_now() {
        let json = r#"{"value": 0}"#;
        let before = Timestamp::now();
        let result: TestTimestampMs = serde_json::from_str(json).expect("should deserialize zero");
        let after = Timestamp::now();

        assert!(
            result.value >= before && result.value <= after,
            "Zero timestamp should return approximately Timestamp::now()"
        );
    }

    #[test]
    fn test_timestamp_ms_valid_converts_to_exact_instant() {
        // Use a known timestamp: 2024-01-01 00:00:00 UTC = 1704067200000 ms
        let json = r#"{"value": 1704067200000}"#;
        let result: TestTimestampMs =
            serde_json::from_str(json).expect("should deserialize valid timestamp");

        assert_eq!(result.value.as_millisecond(), 1_704_067_200_000);
        assert_eq!(result.value.as_second(), 1_704_067_200);
    }

    #[test]
    fn test_timestamp_ms_overflow_fails() {
        // u64::MAX exceeds i64::MAX, so the conversion must reject it.
        let json = format!(r#"{{"value": {}}}"#, u64::MAX);
        let result: Result<TestTimestampMs, _> = serde_json::from_str(&json);
        assert!(
            result.is_err(),
            "u64::MAX milliseconds should fail deserialization"
        );
    }

    #[test]
    fn test_timestamp_ms_future_works() {
        // Use a future timestamp (current time + 1 hour in ms)
        let future_ms = Timestamp::now().as_millisecond() + 3_600_000;
        let json = format!(r#"{{"value": {future_ms}}}"#);
        let result: TestTimestampMs =
            serde_json::from_str(&json).expect("should deserialize future timestamp");

        // Should be approximately 1 hour in the future
        let diff = result.value.duration_since(Timestamp::now()).as_secs();
        assert!(
            (3590..=3610).contains(&diff),
            "Future timestamp should be ~1 hour from now, got diff: {diff}s"
        );
    }

    // =========================================================================
    // Serialization contract — locks the JSON shape the TS frontend parses
    // =========================================================================

    #[test]
    fn test_timestamp_serializes_as_rfc3339_utc_z() {
        // jiff::Timestamp serializes to RFC3339 with a trailing `Z` (UTC). The
        // frontend parses this with `new Date(...)`; this test pins the shape so
        // a future serde change can't silently break the contract.
        let ts = Timestamp::from_second(1_704_067_200).expect("valid timestamp");
        let json = serde_json::to_string(&ts).expect("serialize");
        assert_eq!(json, r#""2024-01-01T00:00:00Z""#);

        let back: Timestamp = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, ts);
    }
}
