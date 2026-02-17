//! Secrets configuration for sensitive credentials.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Secrets configuration loaded from `secrets.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// `HoYoLab` authentication.
    #[serde(default)]
    pub hoyolab: HoyolabSecrets,

    /// Kuro Games authentication.
    #[serde(default)]
    pub kuro: KuroSecrets,
}

impl SecretsConfig {
    /// Loads secrets from the default secrets file location.
    ///
    /// # Errors
    ///
    /// Returns an error if the secrets file cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let path = Self::secrets_path()?;
        Self::load_from_path(&path)
    }

    /// Loads secrets from a specific path.
    ///
    /// # Errors
    ///
    /// Returns an error if the secrets file cannot be read or parsed.
    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(Error::ConfigNotFound {
                path: path.display().to_string(),
            });
        }

        let content = std::fs::read_to_string(path)?;
        let secrets: Self = toml::from_str(&content)?;
        Ok(secrets)
    }

    /// Returns the default secrets file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be determined.
    pub fn secrets_path() -> Result<PathBuf> {
        Ok(super::AppConfig::config_dir()?.join("secrets.toml"))
    }

    /// Saves the secrets to the default secrets file location.
    ///
    /// # Errors
    ///
    /// Returns an error if the secrets file cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = Self::secrets_path()?;
        self.save_to_path(&path)
    }

    /// Saves the secrets to a specific path.
    ///
    /// # Errors
    ///
    /// Returns an error if the secrets file cannot be written.
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| Error::ConfigParseFailed {
            message: format!("Failed to serialize secrets: {e}"),
        })?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Creates an empty secrets file if it doesn't exist.
    ///
    /// Returns `true` if a new file was created, `false` if it already existed.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written.
    pub fn create_default_if_missing() -> Result<bool> {
        let path = Self::secrets_path()?;

        if path.exists() {
            return Ok(false);
        }

        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write empty secrets with helpful comments
        let content = Self::default_secrets_content();
        std::fs::write(&path, content)?;

        // Verify it can be loaded
        let _ = Self::load_from_path(&path)?;

        tracing::info!("Created default secrets file at: {}", path.display());
        Ok(true)
    }

    /// Returns the default secrets file content with helpful comments.
    fn default_secrets_content() -> &'static str {
        r#"# Storekeeper Secrets Configuration
# This file contains sensitive credentials. Keep it secure!
#
# WARNING: Never share this file or commit it to version control.

# =============================================================================
# HoYoLab Authentication (for Genshin Impact, HSR, ZZZ)
# =============================================================================
# Get these cookies from the HoYoLab website:
# 1. Go to https://www.hoyolab.com and log in
# 2. Open browser Developer Tools (F12) > Application > Cookies
# 3. Find cookies named "ltuid_v2", "ltoken_v2", and "ltmid_v2"

[hoyolab]
# Required: v2 authentication cookies
ltuid_v2 = ""
ltoken_v2 = ""
ltmid_v2 = ""

# =============================================================================
# Kuro Games Authentication (for Wuthering Waves)
# =============================================================================
# AUTOMATIC: The oauth_code is automatically loaded from the Kuro SDK cache:
#   %APPDATA%\KR_G153\A1730\KRSDKUserLauncherCache.json
#
# You typically don't need to set anything here - just enable Wuthering Waves
# in config.toml and credentials will be loaded automatically.
#
# OPTIONAL OVERRIDE: Set oauth_code below to override automatic detection.

[kuro]
# Optional: uncomment and set to override automatic detection
# oauth_code = ""
"#
    }
}

/// `HoYoLab` authentication secrets.
///
/// Uses the v2 cookie format which is the current standard on HoYoLab.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HoyolabSecrets {
    /// `HoYoLab` account ID v2 (ltuid_v2 cookie).
    #[serde(default)]
    pub ltuid_v2: String,

    /// `HoYoLab` token v2 (ltoken_v2 cookie).
    #[serde(default)]
    pub ltoken_v2: String,

    /// `HoYoLab` mid token v2 (ltmid_v2 cookie).
    #[serde(default)]
    pub ltmid_v2: String,
}

impl HoyolabSecrets {
    /// Checks if the required authentication fields are set.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.ltuid_v2.is_empty() && !self.ltoken_v2.is_empty()
    }

    /// Returns the ltuid value for API requests.
    #[must_use]
    pub fn ltuid(&self) -> &str {
        &self.ltuid_v2
    }

    /// Returns the ltoken value for API requests.
    #[must_use]
    pub fn ltoken(&self) -> &str {
        &self.ltoken_v2
    }
}

/// Kuro Games authentication secrets.
///
/// The `oauth_code` field is optional - if not provided in `secrets.toml`,
/// the application will attempt to load it automatically from the Kuro SDK
/// launcher cache at `%APPDATA%/KR_G153/A1730/KRSDKUserLauncherCache.json`.
///
/// If set in `secrets.toml`, it acts as an override for the cached value.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KuroSecrets {
    /// OAuth code (XOR-5 decoded) from launcher cache.
    /// Optional: if not provided, will be loaded from the Kuro SDK cache.
    #[serde(default)]
    pub oauth_code: String,
}

impl KuroSecrets {
    /// Checks if an OAuth code override is set in secrets.
    #[must_use]
    pub fn has_override(&self) -> bool {
        !self.oauth_code.is_empty()
    }

    /// Returns the OAuth code override if set.
    #[must_use]
    pub fn oauth_code_override(&self) -> Option<&str> {
        if self.oauth_code.is_empty() {
            None
        } else {
            Some(&self.oauth_code)
        }
    }
}
