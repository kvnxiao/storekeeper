//! Configuration types for Storekeeper.
//!
//! Configuration is split into two files:
//! - `config.toml`: Non-sensitive settings that can be synced across machines
//! - `secrets.toml`: Sensitive credentials that must be set manually

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::region::Region;

/// Main application configuration loaded from `config.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    /// General application settings.
    #[serde(default)]
    pub general: GeneralConfig,

    /// Notification settings.
    #[serde(default)]
    pub notifications: NotificationConfig,

    /// Per-game configuration.
    #[serde(default)]
    pub games: GamesConfig,
}

impl AppConfig {
    /// Loads configuration from the default config file location.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        Self::load_from_path(&path)
    }

    /// Loads configuration from a specific path.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file cannot be read or parsed.
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(Error::ConfigNotFound {
                path: path.display().to_string(),
            });
        }

        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Returns the default config file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be determined.
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| Error::ConfigNotFound {
            path: "config directory".to_string(),
        })?;
        Ok(config_dir.join("storekeeper").join("config.toml"))
    }

    /// Returns the config directory path.
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be determined.
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| Error::ConfigNotFound {
            path: "config directory".to_string(),
        })?;
        Ok(config_dir.join("storekeeper"))
    }

    /// Creates a default configuration file if it doesn't exist.
    ///
    /// Returns `true` if a new file was created, `false` if it already existed.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written.
    pub fn create_default_if_missing() -> Result<bool> {
        let path = Self::config_path()?;

        if path.exists() {
            return Ok(false);
        }

        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write default config with helpful comments
        let content = Self::default_config_content();
        std::fs::write(&path, content)?;

        // Verify it can be loaded
        let _ = Self::load_from_path(&path)?;

        tracing::info!("Created default config file at: {}", path.display());
        Ok(true)
    }

    /// Returns the default config file content with helpful comments.
    fn default_config_content() -> &'static str {
        r#"# Storekeeper Configuration
# This file contains non-sensitive application settings.
#
# For sensitive credentials (API tokens, cookies), see secrets.toml

[general]
# Polling interval in seconds (default: 300 = 5 minutes)
poll_interval_secs = 300

# Start the app minimized to system tray (default: true)
start_minimized = true

# Log level: error, warn, info, debug, trace (default: info)
log_level = "info"

[notifications]
# Enable desktop notifications (default: true)
enabled = true

# Cooldown between notifications in minutes (default: 30)
cooldown_minutes = 30

# Per-resource notification thresholds
# Uncomment and customize as needed
#
# [notifications.thresholds.resin]
# enabled = true
# threshold_value = 150
# notify_when_full = true

# =============================================================================
# GAME CONFIGURATION
# =============================================================================
# Enable only the games you play. Each game requires:
# 1. enabled = true
# 2. Your UID/Player ID
# 3. Credentials in secrets.toml

# Genshin Impact
[games.genshin_impact]
enabled = false
uid = "YOUR_UID_HERE"
# region = "os_usa"  # Optional: auto-detected from UID

# Honkai: Star Rail
[games.honkai_star_rail]
enabled = false
uid = "YOUR_UID_HERE"
# region = "prod_official_usa"  # Optional: auto-detected from UID

# Zenless Zone Zero
[games.zenless_zone_zero]
enabled = false
uid = "YOUR_UID_HERE"
# region = "prod_gf_us"  # Optional: auto-detected from UID

# Wuthering Waves
[games.wuthering_waves]
enabled = false
player_id = "YOUR_PLAYER_ID_HERE"
# region = "na"  # Optional: auto-detected from player ID
"#
    }
}

/// General application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Polling interval in seconds.
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,

    /// Whether to start minimized to tray.
    #[serde(default = "default_true")]
    pub start_minimized: bool,

    /// Log level.
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: default_poll_interval(),
            start_minimized: true,
            log_level: default_log_level(),
        }
    }
}

fn default_poll_interval() -> u64 {
    300
}

fn default_true() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

/// Notification configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Whether notifications are enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Cooldown between notifications in minutes.
    #[serde(default = "default_notification_cooldown")]
    pub cooldown_minutes: u64,

    /// Per-resource notification thresholds.
    #[serde(default)]
    pub thresholds: HashMap<String, ThresholdConfig>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cooldown_minutes: default_notification_cooldown(),
            thresholds: HashMap::new(),
        }
    }
}

fn default_notification_cooldown() -> u64 {
    30
}

/// Threshold configuration for a specific resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Whether this threshold is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// The value at which to trigger a notification.
    pub threshold_value: u32,

    /// Whether to notify when the resource is full.
    #[serde(default = "default_true")]
    pub notify_when_full: bool,
}

/// Per-game configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GamesConfig {
    /// Genshin Impact configuration.
    pub genshin_impact: Option<GenshinConfig>,

    /// Honkai: Star Rail configuration.
    pub honkai_star_rail: Option<HsrConfig>,

    /// Zenless Zone Zero configuration.
    pub zenless_zone_zero: Option<ZzzConfig>,

    /// Wuthering Waves configuration.
    pub wuthering_waves: Option<WuwaConfig>,
}

/// Genshin Impact specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenshinConfig {
    /// Whether this game is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Player UID.
    pub uid: String,

    /// Optional region override.
    pub region: Option<Region>,

    /// Resources to track.
    #[serde(default = "default_genshin_resources")]
    pub tracked_resources: Vec<String>,
}

fn default_genshin_resources() -> Vec<String> {
    vec![
        "resin".to_string(),
        "parametric_transformer".to_string(),
        "realm_currency".to_string(),
        "expeditions".to_string(),
    ]
}

/// Honkai: Star Rail specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsrConfig {
    /// Whether this game is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Player UID.
    pub uid: String,

    /// Optional region override.
    pub region: Option<Region>,

    /// Resources to track.
    #[serde(default = "default_hsr_resources")]
    pub tracked_resources: Vec<String>,
}

fn default_hsr_resources() -> Vec<String> {
    vec!["trailblaze_power".to_string()]
}

/// Zenless Zone Zero specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZzzConfig {
    /// Whether this game is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Player UID.
    pub uid: String,

    /// Optional region override.
    pub region: Option<Region>,

    /// Resources to track.
    #[serde(default = "default_zzz_resources")]
    pub tracked_resources: Vec<String>,
}

fn default_zzz_resources() -> Vec<String> {
    vec!["battery".to_string()]
}

/// Wuthering Waves specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WuwaConfig {
    /// Whether this game is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Player ID.
    pub player_id: String,

    /// Optional region override.
    pub region: Option<Region>,

    /// Resources to track.
    #[serde(default = "default_wuwa_resources")]
    pub tracked_resources: Vec<String>,
}

fn default_wuwa_resources() -> Vec<String> {
    vec!["waveplates".to_string()]
}

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
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
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
        let config_dir = dirs::config_dir().ok_or_else(|| Error::ConfigNotFound {
            path: "config directory".to_string(),
        })?;
        Ok(config_dir.join("storekeeper").join("secrets.toml"))
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

/// Ensures both config files exist, creating defaults if missing.
///
/// This should be called at application startup to ensure the user
/// has template configuration files to edit.
///
/// # Errors
///
/// Returns an error if the config directory cannot be created or
/// the files cannot be written.
pub fn ensure_configs_exist() -> Result<()> {
    let config_created = AppConfig::create_default_if_missing()?;
    let secrets_created = SecretsConfig::create_default_if_missing()?;

    if config_created || secrets_created {
        if let Ok(config_dir) = AppConfig::config_dir() {
            tracing::info!("Configuration files created in: {}", config_dir.display());
        }
    }

    Ok(())
}
