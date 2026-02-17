//! Configuration types for Storekeeper.
//!
//! Configuration is split into two files:
//! - `config.toml`: Non-sensitive settings that can be synced across machines
//! - `secrets.toml`: Sensitive credentials that must be set manually

pub mod claim_time;
pub mod games;
pub mod notification;
pub mod secrets;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

// Re-exports: keep the same public surface as the original single-file module.
pub use claim_time::{ClaimTime, DEFAULT_AUTO_CLAIM_TIME, next_claim_datetime_utc};
pub use games::{GenshinConfig, HsrConfig, WuwaConfig, ZzzConfig};
pub use notification::ResourceNotificationConfig;
pub use secrets::SecretsConfig;

// ============================================================================
// Shared serde default functions
// ============================================================================

pub(crate) fn default_true() -> bool {
    true
}

fn default_poll_interval() -> u64 {
    300
}

fn default_log_level() -> String {
    "info".to_string()
}

// ============================================================================
// AppConfig
// ============================================================================

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
    pub fn load_from_path(path: &Path) -> Result<Self> {
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
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
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

# Automatically start the app when the system boots (default: false)
autostart = false

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
#   notify_minutes_before_full = 60  # Start notifying 60 min before full
#   # notify_at_value = 180          # OR: notify when value reaches 180 (stamina resources only)
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

// ============================================================================
// GeneralConfig
// ============================================================================

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

    /// Language/locale override for the application (e.g. "en", "zh-CN").
    /// When `None`, the system locale is auto-detected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Whether to automatically start the app at system login.
    #[serde(default)]
    pub autostart: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: default_poll_interval(),
            start_minimized: true,
            log_level: default_log_level(),
            language: None,
            autostart: false,
        }
    }
}

// ============================================================================
// GamesConfig
// ============================================================================

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

impl GamesConfig {
    /// Notification configs for a game, with string keys for the notification system.
    ///
    /// Converts typed resource keys to strings via `AsRef<str>`.
    #[must_use]
    pub fn notification_configs(
        &self,
        game_id: crate::GameId,
    ) -> std::collections::HashMap<String, ResourceNotificationConfig> {
        use crate::GameId;

        match game_id {
            GameId::GenshinImpact => self
                .genshin_impact
                .as_ref()
                .map(|c| {
                    c.notifications
                        .iter()
                        .map(|(k, v)| (k.as_ref().to_string(), v.clone()))
                        .collect()
                })
                .unwrap_or_default(),
            GameId::HonkaiStarRail => self
                .honkai_star_rail
                .as_ref()
                .map(|c| {
                    c.notifications
                        .iter()
                        .map(|(k, v)| (k.as_ref().to_string(), v.clone()))
                        .collect()
                })
                .unwrap_or_default(),
            GameId::ZenlessZoneZero => self
                .zenless_zone_zero
                .as_ref()
                .map(|c| {
                    c.notifications
                        .iter()
                        .map(|(k, v)| (k.as_ref().to_string(), v.clone()))
                        .collect()
                })
                .unwrap_or_default(),
            GameId::WutheringWaves => self
                .wuthering_waves
                .as_ref()
                .map(|c| {
                    c.notifications
                        .iter()
                        .map(|(k, v)| (k.as_ref().to_string(), v.clone()))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    /// Whether a game is enabled in config.
    #[must_use]
    pub fn is_enabled(&self, game_id: crate::GameId) -> bool {
        use crate::GameId;

        match game_id {
            GameId::GenshinImpact => self.genshin_impact.as_ref().is_some_and(|c| c.enabled),
            GameId::HonkaiStarRail => self.honkai_star_rail.as_ref().is_some_and(|c| c.enabled),
            GameId::ZenlessZoneZero => self.zenless_zone_zero.as_ref().is_some_and(|c| c.enabled),
            GameId::WutheringWaves => self.wuthering_waves.as_ref().is_some_and(|c| c.enabled),
        }
    }

    /// Whether auto-claim is enabled for a game.
    ///
    /// Wuthering Waves does not support daily rewards, so always returns `false`.
    #[must_use]
    pub fn auto_claim_enabled(&self, game_id: crate::GameId) -> bool {
        use crate::GameId;

        match game_id {
            GameId::GenshinImpact => self
                .genshin_impact
                .as_ref()
                .is_some_and(|c| c.enabled && c.auto_claim_daily_rewards),
            GameId::HonkaiStarRail => self
                .honkai_star_rail
                .as_ref()
                .is_some_and(|c| c.enabled && c.auto_claim_daily_rewards),
            GameId::ZenlessZoneZero => self
                .zenless_zone_zero
                .as_ref()
                .is_some_and(|c| c.enabled && c.auto_claim_daily_rewards),
            GameId::WutheringWaves => false,
        }
    }

    /// Auto-claim time for a game.
    #[must_use]
    pub fn auto_claim_time(&self, game_id: crate::GameId) -> Option<ClaimTime> {
        use crate::GameId;

        match game_id {
            GameId::GenshinImpact => self.genshin_impact.as_ref().and_then(|c| c.auto_claim_time),
            GameId::HonkaiStarRail => self
                .honkai_star_rail
                .as_ref()
                .and_then(|c| c.auto_claim_time),
            GameId::ZenlessZoneZero => self
                .zenless_zone_zero
                .as_ref()
                .and_then(|c| c.auto_claim_time),
            GameId::WutheringWaves => None,
        }
    }
}

// ============================================================================
// ensure_configs_exist
// ============================================================================

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
