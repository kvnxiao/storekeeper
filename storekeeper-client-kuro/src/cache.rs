//! Kuro SDK launcher cache file loading.
//!
//! This module provides functionality to load the OAuth code from the
//! Kuro Games launcher's local cache file.

use std::path::PathBuf;

use serde::Deserialize;

use crate::error::{ClientError, Error, Result};

/// The expected path relative to AppData/Roaming for the Kuro SDK cache.
const KURO_SDK_CACHE_PATH: &str = "KR_G153/A1730/KRSDKUserLauncherCache.json";

/// Structure of a user entry in the Kuro SDK launcher cache JSON file.
///
/// The cache file is a JSON array of user objects.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KuroSdkCacheEntry {
    /// The XOR-5 encoded OAuth code.
    oauth_code: Option<String>,
}

/// Decodes an XOR-5 encoded string.
fn decode_xor5(s: &str) -> String {
    s.chars().map(|c| char::from((c as u8) ^ 5)).collect()
}

/// Attempts to load the OAuth code from the Kuro SDK launcher cache.
///
/// This function looks for the cache file at:
/// - Windows: `%APPDATA%/KR_G153/A1730/KRSDKUserLauncherCache.json`
///
/// The OAuth code in the cache file is XOR-5 encoded and will be decoded
/// before being returned.
///
/// # Returns
///
/// Returns `Ok(Some(oauth_code))` if the cache file exists and contains a valid
/// OAuth code. Returns `Ok(None)` if the cache file doesn't exist or is empty.
/// Returns `Err` if there's an error reading or parsing the file.
///
/// # Errors
///
/// Returns an error if:
/// - The roaming app data directory cannot be determined
/// - The cache file exists but cannot be read
/// - The cache file contains invalid JSON
pub fn load_oauth_from_cache() -> Result<Option<String>> {
    let cache_path = get_cache_path()?;

    if !cache_path.exists() {
        tracing::debug!("Kuro SDK cache file not found at: {}", cache_path.display());
        return Ok(None);
    }

    tracing::debug!("Loading Kuro SDK cache from: {}", cache_path.display());

    let content = std::fs::read_to_string(&cache_path).map_err(|e| {
        Error::Client(ClientError::invalid_config(format!(
            "Failed to read Kuro SDK cache file at {}: {}",
            cache_path.display(),
            e
        )))
    })?;

    let cache_entries: Vec<KuroSdkCacheEntry> = serde_json::from_str(&content).map_err(|e| {
        Error::Client(ClientError::invalid_config(format!(
            "Failed to parse Kuro SDK cache file at {}: {}",
            cache_path.display(),
            e
        )))
    })?;

    // Find the first entry with a non-empty OAuth code
    for entry in cache_entries {
        if let Some(encoded) = entry.oauth_code {
            if !encoded.is_empty() {
                let decoded = decode_xor5(&encoded);
                tracing::info!("Successfully loaded OAuth code from Kuro SDK cache");
                return Ok(Some(decoded));
            }
        }
    }

    tracing::debug!("Kuro SDK cache file exists but contains no OAuth code");
    Ok(None)
}

/// Returns the path to the Kuro SDK cache file.
///
/// # Errors
///
/// Returns an error if the roaming app data directory cannot be determined.
fn get_cache_path() -> Result<PathBuf> {
    // On Windows, dirs::data_dir() returns %APPDATA% (Roaming)
    let data_dir = dirs::data_dir().ok_or_else(|| {
        Error::Client(ClientError::invalid_config(
            "Could not determine roaming app data directory",
        ))
    })?;

    Ok(data_dir.join(KURO_SDK_CACHE_PATH))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_xor5() {
        // XOR-5 is its own inverse
        let original = "test_oauth_code_123";
        let encoded: String = original
            .chars()
            .map(|c| char::from((c as u8) ^ 5))
            .collect();
        let decoded = decode_xor5(&encoded);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_decode_xor5_roundtrip() {
        let test_strings = ["", "a", "hello", "OAuth123!@#", "日本語"]; // Note: non-ASCII may have issues
        for original in test_strings {
            // Only test ASCII strings as XOR-5 on bytes may produce invalid UTF-8 for non-ASCII
            if original.is_ascii() {
                let encoded: String = original
                    .chars()
                    .map(|c| char::from((c as u8) ^ 5))
                    .collect();
                let decoded = decode_xor5(&encoded);
                assert_eq!(decoded, original, "Failed roundtrip for: {original}");
            }
        }
    }
}
