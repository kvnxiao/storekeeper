//! Configuration types for Storekeeper.
//!
//! Configuration is split into two files:
//! - `config.toml`: Non-sensitive settings that can be synced across machines
//! - `secrets.toml`: Sensitive credentials that must be set manually

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::region::Region;

// =============================================================================
// ClaimTime type
// =============================================================================

/// UTC+8 offset in seconds (8 hours = 28800 seconds).
const UTC8_OFFSET_SECS: i32 = 8 * 3600;

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

// =============================================================================
// Serde module for Option<ClaimTime>
// =============================================================================

/// Serde module for serializing/deserializing `Option<ClaimTime>`.
///
/// - **Deserialize:** Parses "HH:MM" as UTC+8, converts to UTC for storage.
/// - **Serialize:** Converts UTC to UTC+8, formats as "HH:MM".
mod claim_time_serde {
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

/// Main application configuration loaded from `config.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    /// General application settings.
    #[serde(default)]
    pub general: GeneralConfig,

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

    /// Saves the configuration to the default config file location.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        self.save_to_path(&path)
    }

    /// Saves the configuration to a specific path.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file cannot be written.
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| Error::ConfigParseFailed {
            message: format!("Failed to serialize config: {e}"),
        })?;
        std::fs::write(path, content)?;
        Ok(())
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

# =============================================================================
# GAME CONFIGURATION
# =============================================================================
# Enable only the games you play. Each game requires:
# 1. enabled = true
# 2. Your UID/Player ID
# 3. Credentials in secrets.toml
#
# HoYoLab games (Genshin, HSR, ZZZ) support auto-claiming daily rewards:
#   auto_claim_daily_rewards = true/false
#   auto_claim_time = "HH:MM"  # Optional, in UTC+8. Defaults to "00:00" (midnight)
#
# Per-resource notifications (optional):
#   [games.<game>.notifications.<resource_type>]
#   enabled = true
#   notify_minutes_before_full = 60  # Start notifying 60 min before full (0 = only when full)
#   cooldown_minutes = 10            # Minutes between repeated notifications

# Genshin Impact
[games.genshin_impact]
enabled = false
uid = "YOUR_UID_HERE"
# region = "os_usa"  # Optional: auto-detected from UID
# auto_claim_daily_rewards = false
# auto_claim_time = "00:00"  # Optional: HH:MM in UTC+8 (China Standard Time)
#
# [games.genshin_impact.notifications.resin]
# enabled = true
# notify_minutes_before_full = 60
# cooldown_minutes = 10

# Honkai: Star Rail
[games.honkai_star_rail]
enabled = false
uid = "YOUR_UID_HERE"
# region = "prod_official_usa"  # Optional: auto-detected from UID
# auto_claim_daily_rewards = false
# auto_claim_time = "00:00"  # Optional: HH:MM in UTC+8 (China Standard Time)
#
# [games.honkai_star_rail.notifications.trailblaze_power]
# enabled = true
# notify_minutes_before_full = 30
# cooldown_minutes = 15

# Zenless Zone Zero
[games.zenless_zone_zero]
enabled = false
uid = "YOUR_UID_HERE"
# region = "prod_gf_us"  # Optional: auto-detected from UID
# auto_claim_daily_rewards = false
# auto_claim_time = "00:00"  # Optional: HH:MM in UTC+8 (China Standard Time)
#
# [games.zenless_zone_zero.notifications.battery]
# enabled = true
# notify_minutes_before_full = 30
# cooldown_minutes = 15

# Wuthering Waves
[games.wuthering_waves]
enabled = false
player_id = "YOUR_PLAYER_ID_HERE"
# region = "na"  # Optional: auto-detected from player ID
#
# [games.wuthering_waves.notifications.waveplates]
# enabled = true
# notify_minutes_before_full = 30
# cooldown_minutes = 15
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

/// Notification configuration for a specific resource.
///
/// Controls when and how often OS notifications are sent for a tracked resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNotificationConfig {
    /// Whether notifications are enabled for this resource.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Minutes before full to start notifying. 0 = only when full/ready.
    #[serde(default)]
    pub notify_minutes_before_full: u32,

    /// Minutes between repeated notifications.
    #[serde(default = "default_notification_cooldown")]
    pub cooldown_minutes: u32,
}

fn default_notification_cooldown() -> u32 {
    30
}

/// Default claim time in UTC+8 (midnight), displayed as "00:00".
pub const DEFAULT_AUTO_CLAIM_TIME: &str = "00:00";

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

    /// Whether to auto-claim daily rewards for this game.
    #[serde(default)]
    pub auto_claim_daily_rewards: bool,

    /// Optional time to auto-claim daily rewards in HH:MM format (UTC+8).
    /// Internally stored as UTC. If not specified, defaults to "00:00" (midnight UTC+8).
    #[serde(default, with = "claim_time_serde")]
    pub auto_claim_time: Option<ClaimTime>,

    /// Per-resource notification settings.
    #[serde(default)]
    pub notifications: HashMap<String, ResourceNotificationConfig>,
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

    /// Whether to auto-claim daily rewards for this game.
    #[serde(default)]
    pub auto_claim_daily_rewards: bool,

    /// Optional time to auto-claim daily rewards in HH:MM format (UTC+8).
    /// Internally stored as UTC. If not specified, defaults to "00:00" (midnight UTC+8).
    #[serde(default, with = "claim_time_serde")]
    pub auto_claim_time: Option<ClaimTime>,

    /// Per-resource notification settings.
    #[serde(default)]
    pub notifications: HashMap<String, ResourceNotificationConfig>,
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

    /// Whether to auto-claim daily rewards for this game.
    #[serde(default)]
    pub auto_claim_daily_rewards: bool,

    /// Optional time to auto-claim daily rewards in HH:MM format (UTC+8).
    /// Internally stored as UTC. If not specified, defaults to "00:00" (midnight UTC+8).
    #[serde(default, with = "claim_time_serde")]
    pub auto_claim_time: Option<ClaimTime>,

    /// Per-resource notification settings.
    #[serde(default)]
    pub notifications: HashMap<String, ResourceNotificationConfig>,
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

    /// Per-resource notification settings.
    #[serde(default)]
    pub notifications: HashMap<String, ResourceNotificationConfig>,
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
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

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

    #[test]
    fn test_backward_compatibility_with_game_config() {
        // Simulates old config files that users have
        let toml_str = r#"
            enabled = true
            uid = "123456789"
            auto_claim_daily_rewards = true
            auto_claim_time = "08:30"
        "#;

        let config: GenshinConfig = toml::from_str(toml_str).expect("should parse config");
        assert!(config.enabled);
        assert_eq!(config.uid, "123456789");
        assert!(config.auto_claim_daily_rewards);

        let time = config.auto_claim_time.expect("should have time");
        assert_eq!(time.to_utc8_string(), "08:30");
    }

    #[test]
    fn test_game_config_without_claim_time() {
        // Config without auto_claim_time should have None
        let toml_str = r#"
            enabled = true
            uid = "123456789"
        "#;

        let config: GenshinConfig = toml::from_str(toml_str).expect("should parse config");
        assert!(config.auto_claim_time.is_none());
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

    // =========================================================================
    // ResourceNotificationConfig tests
    // =========================================================================

    #[test]
    fn test_resource_notification_config_serde_roundtrip() {
        let toml_str = r"
            enabled = true
            notify_minutes_before_full = 60
            cooldown_minutes = 10
        ";

        let config: ResourceNotificationConfig =
            toml::from_str(toml_str).expect("should parse notification config");
        assert!(config.enabled);
        assert_eq!(config.notify_minutes_before_full, 60);
        assert_eq!(config.cooldown_minutes, 10);

        let serialized = toml::to_string(&config).expect("should serialize");
        let roundtripped: ResourceNotificationConfig =
            toml::from_str(&serialized).expect("should roundtrip");
        assert_eq!(roundtripped.enabled, config.enabled);
        assert_eq!(
            roundtripped.notify_minutes_before_full,
            config.notify_minutes_before_full
        );
        assert_eq!(roundtripped.cooldown_minutes, config.cooldown_minutes);
    }

    #[test]
    fn test_game_config_with_notifications() {
        let toml_str = r#"
            enabled = true
            uid = "123456789"

            [notifications.resin]
            enabled = true
            notify_minutes_before_full = 60
            cooldown_minutes = 10

            [notifications.expeditions]
            enabled = true
            notify_minutes_before_full = 0
            cooldown_minutes = 30
        "#;

        let config: GenshinConfig = toml::from_str(toml_str).expect("should parse config");
        assert_eq!(config.notifications.len(), 2);

        let resin = config
            .notifications
            .get("resin")
            .expect("should have resin config");
        assert!(resin.enabled);
        assert_eq!(resin.notify_minutes_before_full, 60);
        assert_eq!(resin.cooldown_minutes, 10);

        let expeditions = config
            .notifications
            .get("expeditions")
            .expect("should have expeditions config");
        assert!(expeditions.enabled);
        assert_eq!(expeditions.notify_minutes_before_full, 0);
        assert_eq!(expeditions.cooldown_minutes, 30);
    }

    #[test]
    fn test_game_config_without_notifications_backward_compat() {
        // Old config without notifications field should still parse
        let toml_str = r#"
            enabled = true
            uid = "123456789"
            auto_claim_daily_rewards = true
        "#;

        let config: GenshinConfig = toml::from_str(toml_str).expect("should parse config");
        assert!(config.notifications.is_empty());
    }

    #[test]
    fn test_app_config_without_notifications_section() {
        // AppConfig no longer has a top-level notifications section
        let toml_str = r"
            [general]
            poll_interval_secs = 300
        ";

        let config: AppConfig = toml::from_str(toml_str).expect("should parse config");
        assert_eq!(config.general.poll_interval_secs, 300);
    }
}
