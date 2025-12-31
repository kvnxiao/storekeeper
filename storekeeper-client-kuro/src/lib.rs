//! Kuro Games API client for Storekeeper.
//!
//! This crate provides an HTTP client for interacting with the Kuro Games API,
//! used by Wuthering Waves.

pub mod cache;
pub mod client;
pub mod error;

pub use cache::load_oauth_from_cache;
pub use client::KuroClient;
pub use error::{Error, Result};

/// Decodes an XOR-5 encoded string.
///
/// The OAuth code from the Kuro launcher cache is XOR-encoded with 5.
#[must_use]
pub fn decode_xor5(s: &str) -> String {
    s.chars().map(|c| char::from((c as u8) ^ 5)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor5_decode() {
        // XOR-5 is its own inverse
        let original = "test123";
        let encoded: String = original
            .chars()
            .map(|c| char::from((c as u8) ^ 5))
            .collect();
        let decoded = decode_xor5(&encoded);
        assert_eq!(decoded, original);
    }
}
