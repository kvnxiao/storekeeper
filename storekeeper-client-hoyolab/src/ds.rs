//! DS (Dynamic Secret) header generation for HoYoLab API.
//!
//! The DS header is required for authenticated HoYoLab API requests.
//! It consists of a timestamp, random string, and MD5 hash.

use std::fmt::Write;

use md5::{Digest, Md5};
use rand::Rng;

/// Salt for overseas (global) HoYoLab API.
const SALT_OVERSEAS: &str = "6s25p5ox5y14umn1p61aqyyvbvvl3lrt";

/// Salt for Chinese HoYoLab API (miyoushe).
///
/// Currently unused — reserved for future Chinese server (miyoushe) support.
#[allow(dead_code)]
const SALT_CHINESE: &str = "xV8v4Qu54lUKrEYFZkJhB8cuOh9Asafs";

/// Generates a DS header for overseas (global) HoYoLab API.
///
/// Format: `{timestamp},{random},{hash}`
/// - timestamp: Unix timestamp (seconds)
/// - random: 6 random ASCII letters
/// - hash: MD5("salt={salt}&t={t}&r={r}")
#[must_use]
pub fn generate_dynamic_secret_overseas() -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let random = generate_random_lowercase_string(6);
    let hash = compute_md5_overseas(timestamp, &random);

    format!("{timestamp},{random},{hash}")
}

/// Generates a DS header for Chinese HoYoLab API (miyoushe).
///
/// Format: `{timestamp},{random},{hash}`
/// - timestamp: Unix timestamp (seconds)
/// - random: Random integer between 100001 and 200000
/// - hash: MD5("salt={salt}&t={t}&r={r}&b={body}&q={query}")
///
/// Currently unused — reserved for future Chinese server (miyoushe) support.
#[must_use]
#[allow(dead_code)]
pub fn generate_dynamic_secret_chinese(body: &str, query: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let random = generate_random_int();
    let hash = compute_md5_chinese(timestamp, random, body, query);

    format!("{timestamp},{random},{hash}")
}

/// Generates a random string of lowercase ASCII letters.
fn generate_random_lowercase_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(rand::distributions::Uniform::new_inclusive(b'a', b'z'))
        .take(len)
        .map(char::from)
        .collect()
}

/// Generates a random integer between 100001 and 200000 (inclusive).
fn generate_random_int() -> u32 {
    rand::thread_rng().gen_range(100_001..=200_000)
}

/// Computes MD5 hash for overseas DS header.
fn compute_md5_overseas(timestamp: i64, random: &str) -> String {
    let input = format!("salt={SALT_OVERSEAS}&t={timestamp}&r={random}");
    md5_hex(&input)
}

/// Computes MD5 hash for Chinese DS header.
fn compute_md5_chinese(timestamp: i64, random: u32, body: &str, query: &str) -> String {
    let input = format!("salt={SALT_CHINESE}&t={timestamp}&r={random}&b={body}&q={query}");
    md5_hex(&input)
}

/// Computes MD5 hash and returns as hex string.
fn md5_hex(input: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();

    let mut hex = String::with_capacity(32);
    for byte in result {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // MD5 hash tests
    // =========================================================================

    #[test]
    fn test_md5_empty() {
        let hash = md5_hex("");
        assert_eq!(hash, "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn test_md5_hello() {
        let hash = md5_hex("hello");
        assert_eq!(hash, "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn test_md5_known_values() {
        // Test with various known MD5 hashes
        assert_eq!(md5_hex("world"), "7d793037a0760186574b0282f2f435e7");
        assert_eq!(md5_hex("test"), "098f6bcd4621d373cade4e832627b4f6");
        assert_eq!(
            md5_hex("The quick brown fox jumps over the lazy dog"),
            "9e107d9d372bb6826bd81d3542a419d6"
        );
    }

    #[test]
    fn test_md5_is_lowercase() {
        let hash = md5_hex("test");
        assert!(
            hash.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "MD5 hash should be lowercase hex"
        );
    }

    #[test]
    fn test_md5_length_is_32() {
        let hash = md5_hex("any string here");
        assert_eq!(hash.len(), 32, "MD5 hash should always be 32 characters");
    }

    // =========================================================================
    // Overseas DS header tests
    // =========================================================================

    #[test]
    fn test_ds_format() {
        let ds = generate_dynamic_secret_overseas();
        let parts: Vec<&str> = ds.split(',').collect();
        assert_eq!(parts.len(), 3);
        // First part should be a timestamp (digits only)
        assert!(parts[0].chars().all(|c| c.is_ascii_digit()));
        // Second part should be 6 letters
        assert_eq!(parts[1].len(), 6);
        // Third part should be 32 hex chars
        assert_eq!(parts[2].len(), 32);
    }

    #[test]
    fn test_overseas_ds_timestamp_is_current() {
        let before = chrono::Utc::now().timestamp();
        let ds = generate_dynamic_secret_overseas();
        let after = chrono::Utc::now().timestamp();

        let parts: Vec<&str> = ds.split(',').collect();
        let timestamp: i64 = parts[0].parse().expect("should parse timestamp");

        assert!(
            timestamp >= before && timestamp <= after,
            "Timestamp should be current"
        );
    }

    #[test]
    fn test_overseas_ds_random_is_lowercase_letters() {
        let ds = generate_dynamic_secret_overseas();
        let parts: Vec<&str> = ds.split(',').collect();
        let random = parts[1];

        assert_eq!(random.len(), 6, "Random string should be 6 characters");
        assert!(
            random.chars().all(|c| c.is_ascii_lowercase()),
            "Random string should only contain lowercase letters"
        );
    }

    #[test]
    fn test_overseas_ds_hash_is_valid_hex() {
        let ds = generate_dynamic_secret_overseas();
        let parts: Vec<&str> = ds.split(',').collect();
        let hash = parts[2];

        assert_eq!(hash.len(), 32, "Hash should be 32 characters");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be valid hex"
        );
        assert!(
            hash.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "Hash should be lowercase hex"
        );
    }

    #[test]
    fn test_overseas_ds_different_calls_produce_different_random() {
        // Due to randomness, two calls should (almost always) produce different randoms
        let mut randoms: Vec<String> = Vec::new();
        for _ in 0..10 {
            let ds = generate_dynamic_secret_overseas();
            let parts: Vec<&str> = ds.split(',').collect();
            randoms.push(parts[1].to_string());
        }

        // At least some of them should be different
        let first = &randoms[0];
        let has_different = randoms.iter().any(|r| r != first);
        assert!(
            has_different,
            "Multiple calls should produce different random strings"
        );
    }

    // =========================================================================
    // Chinese DS header tests
    // =========================================================================

    #[test]
    fn test_chinese_ds_format() {
        let ds = generate_dynamic_secret_chinese("", "");
        let parts: Vec<&str> = ds.split(',').collect();

        assert_eq!(parts.len(), 3, "DS should have 3 comma-separated parts");
        // First part should be timestamp
        assert!(
            parts[0].chars().all(|c| c.is_ascii_digit()),
            "First part should be digits (timestamp)"
        );
        // Second part should be integer 100001-200000
        let random: u32 = parts[1].parse().expect("should parse random int");
        assert!(
            (100_001..=200_000).contains(&random),
            "Random should be in range 100001-200000, got {random}"
        );
        // Third part should be 32 hex chars
        assert_eq!(parts[2].len(), 32, "Hash should be 32 characters");
    }

    #[test]
    fn test_chinese_ds_with_body_and_query() {
        let ds = generate_dynamic_secret_chinese(
            r#"{"game_biz":"hk4e_global"}"#,
            "role_id=123456789&server=os_usa",
        );
        let parts: Vec<&str> = ds.split(',').collect();

        assert_eq!(parts.len(), 3);
        assert_eq!(parts[2].len(), 32, "Hash should be 32 characters");
    }

    #[test]
    fn test_chinese_ds_random_in_range() {
        for _ in 0..20 {
            let ds = generate_dynamic_secret_chinese("", "");
            let parts: Vec<&str> = ds.split(',').collect();
            let random: u32 = parts[1].parse().expect("should parse random int");

            assert!(
                (100_001..=200_000).contains(&random),
                "Random should be in range 100001-200000, got {random}"
            );
        }
    }

    #[test]
    fn test_chinese_ds_different_body_produces_different_hash() {
        // Same timestamp and random would produce different hashes with different bodies
        // Since we can't control timestamp/random, we just verify the hash is computed
        let ds1 = generate_dynamic_secret_chinese("body1", "");
        let ds2 = generate_dynamic_secret_chinese("body2", "");

        // They should both be valid format
        let parts1: Vec<&str> = ds1.split(',').collect();
        let parts2: Vec<&str> = ds2.split(',').collect();

        assert_eq!(parts1.len(), 3);
        assert_eq!(parts2.len(), 3);
        assert_eq!(parts1[2].len(), 32);
        assert_eq!(parts2[2].len(), 32);
    }

    // =========================================================================
    // Helper function tests
    // =========================================================================

    #[test]
    fn test_generate_random_lowercase_string_length() {
        let s = generate_random_lowercase_string(6);
        assert_eq!(s.len(), 6);

        let s = generate_random_lowercase_string(10);
        assert_eq!(s.len(), 10);

        let s = generate_random_lowercase_string(0);
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_generate_random_lowercase_string_chars() {
        let s = generate_random_lowercase_string(100);
        assert!(
            s.chars().all(|c| c.is_ascii_lowercase()),
            "All characters should be lowercase ASCII letters"
        );
    }

    #[test]
    fn test_generate_random_int_in_range() {
        for _ in 0..100 {
            let r = generate_random_int();
            assert!(
                (100_001..=200_000).contains(&r),
                "Random int should be in range 100001-200000, got {r}"
            );
        }
    }

    // =========================================================================
    // Compute MD5 helper tests
    // =========================================================================

    #[test]
    fn test_compute_md5_overseas_format() {
        // Test that the overseas MD5 computation works with known format
        let timestamp = 1_704_067_200i64; // 2024-01-01 00:00:00 UTC
        let random = "abcdef";

        let hash = compute_md5_overseas(timestamp, random);
        assert_eq!(hash.len(), 32);

        // The input is "salt={SALT}&t={timestamp}&r={random}"
        // We can verify the format is consistent
        let expected_input = format!("salt={SALT_OVERSEAS}&t={timestamp}&r={random}");
        assert_eq!(md5_hex(&expected_input), hash);
    }

    #[test]
    fn test_compute_md5_chinese_format() {
        let timestamp = 1_704_067_200i64;
        let random = 150_000u32;
        let body = r#"{"test":"value"}"#;
        let query = "param=value";

        let hash = compute_md5_chinese(timestamp, random, body, query);
        assert_eq!(hash.len(), 32);

        // Verify the format
        let expected_input =
            format!("salt={SALT_CHINESE}&t={timestamp}&r={random}&b={body}&q={query}");
        assert_eq!(md5_hex(&expected_input), hash);
    }
}
