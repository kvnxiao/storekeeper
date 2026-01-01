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
#[must_use]
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
}
