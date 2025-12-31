//! DS (Dynamic Secret) header generation for HoYoLab API.
//!
//! The DS header is required for authenticated HoYoLab API requests.
//! It consists of a timestamp, random string, and MD5 hash.

use std::fmt::Write;

use md5::{Digest, Md5};

/// Salt for overseas (global) HoYoLab API.
const SALT_OVERSEAS: &str = "6s25p5ox5y14umn1p61aqyyvbvvl3lrt";

/// Salt for Chinese HoYoLab API (miyoushe).
const SALT_CHINESE: &str = "xV8v4Qu54lUKrEYFZkJhB8cuOh9Asafs";

/// Generates a DS header for overseas (global) HoYoLab API.
///
/// Format: `{timestamp},{random},{hash}`
/// - timestamp: Unix timestamp (seconds)
/// - random: 6 random ASCII letters
/// - hash: MD5("salt={salt}&t={t}&r={r}")
#[must_use]
pub fn generate_ds_overseas() -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let random = generate_random_string(6);
    let hash = compute_md5_overseas(timestamp, &random);

    format!("{timestamp},{random},{hash}")
}

/// Generates a DS header for Chinese HoYoLab API (miyoushe).
///
/// Format: `{timestamp},{random},{hash}`
/// - timestamp: Unix timestamp (seconds)
/// - random: Random integer between 100001 and 200000
/// - hash: MD5("salt={salt}&t={t}&r={r}&b={body}&q={query}")
#[must_use]
pub fn generate_ds_chinese(body: &str, query: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let random = generate_random_int();
    let hash = compute_md5_chinese(timestamp, random, body, query);

    format!("{timestamp},{random},{hash}")
}

/// Generates a random 6-character ASCII string.
fn generate_random_string(len: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Simple pseudo-random generation using system time
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let mut result = String::with_capacity(len);
    let mut state = seed;

    for _ in 0..len {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        #[allow(clippy::cast_possible_truncation)]
        let char_index = ((state >> 16) % 26) as u8;
        result.push((b'a' + char_index) as char);
    }

    result
}

/// Generates a random integer between 100001 and 200000.
fn generate_random_int() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let state = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
    let range = 200_000 - 100_001;
    #[allow(clippy::cast_possible_truncation)]
    let result = 100_001 + ((state >> 16) as u32 % range);
    result
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
    fn test_ds_format() {
        let ds = generate_ds_overseas();
        let parts: Vec<&str> = ds.split(',').collect();
        assert_eq!(parts.len(), 3);
        // First part should be a timestamp (digits only)
        assert!(parts[0].chars().all(|c| c.is_ascii_digit()));
        // Second part should be 6 letters
        assert_eq!(parts[1].len(), 6);
        // Third part should be 32 hex chars
        assert_eq!(parts[2].len(), 32);
    }
}
