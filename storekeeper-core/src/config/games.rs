//! Per-game configuration types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::claim_time::{ClaimTime, claim_time_serde};
use super::default_true;
use super::notification::ResourceNotificationConfig;
use crate::region::Region;

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

#[cfg(test)]
mod tests {
    use super::*;

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
            cooldown_minutes = 30
        "#;

        let config: GenshinConfig = toml::from_str(toml_str).expect("should parse config");
        assert_eq!(config.notifications.len(), 2);

        let resin = config
            .notifications
            .get("resin")
            .expect("should have resin config");
        assert!(resin.enabled);
        assert_eq!(resin.notify_minutes_before_full, Some(60));
        assert_eq!(resin.cooldown_minutes, 10);

        let expeditions = config
            .notifications
            .get("expeditions")
            .expect("should have expeditions config");
        assert!(expeditions.enabled);
        assert_eq!(expeditions.notify_minutes_before_full, None);
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
}
