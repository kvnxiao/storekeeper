//! Per-game configuration types.

use std::collections::HashMap;
use std::hash::Hash;

use serde::de::{DeserializeOwned, Deserializer};
use serde::{Deserialize, Serialize};

use super::claim_time::{ClaimTime, claim_time_serde};
use super::default_true;
use super::notification::ResourceNotificationConfig;
use crate::region::Region;
use crate::resource_types::{
    GenshinResourceType, HsrResourceType, WuwaResourceType, ZzzResourceType,
};

/// Genshin Impact specific configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenshinConfig {
    /// Whether this game is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Player UID.
    pub uid: String,

    /// Optional region override.
    pub region: Option<Region>,

    /// Resources to track.
    #[serde(
        default = "default_genshin_resources",
        deserialize_with = "deserialize_genshin_tracked_resources"
    )]
    pub tracked_resources: Vec<GenshinResourceType>,

    /// Whether to auto-claim daily rewards for this game.
    #[serde(default)]
    pub auto_claim_daily_rewards: bool,

    /// Optional time to auto-claim daily rewards in HH:MM format (UTC+8).
    /// Internally stored as UTC. If not specified, defaults to "00:00" (midnight UTC+8).
    #[serde(default, with = "claim_time_serde")]
    pub auto_claim_time: Option<ClaimTime>,

    /// Per-resource notification settings.
    #[serde(default, deserialize_with = "deserialize_genshin_notifications")]
    pub notifications: HashMap<GenshinResourceType, ResourceNotificationConfig>,
}

fn default_genshin_resources() -> Vec<GenshinResourceType> {
    GenshinResourceType::all().to_vec()
}

/// Honkai: Star Rail specific configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HsrConfig {
    /// Whether this game is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Player UID.
    pub uid: String,

    /// Optional region override.
    pub region: Option<Region>,

    /// Resources to track.
    #[serde(
        default = "default_hsr_resources",
        deserialize_with = "deserialize_hsr_tracked_resources"
    )]
    pub tracked_resources: Vec<HsrResourceType>,

    /// Whether to auto-claim daily rewards for this game.
    #[serde(default)]
    pub auto_claim_daily_rewards: bool,

    /// Optional time to auto-claim daily rewards in HH:MM format (UTC+8).
    /// Internally stored as UTC. If not specified, defaults to "00:00" (midnight UTC+8).
    #[serde(default, with = "claim_time_serde")]
    pub auto_claim_time: Option<ClaimTime>,

    /// Per-resource notification settings.
    #[serde(default, deserialize_with = "deserialize_hsr_notifications")]
    pub notifications: HashMap<HsrResourceType, ResourceNotificationConfig>,
}

fn default_hsr_resources() -> Vec<HsrResourceType> {
    HsrResourceType::all().to_vec()
}

/// Zenless Zone Zero specific configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZzzConfig {
    /// Whether this game is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Player UID.
    pub uid: String,

    /// Optional region override.
    pub region: Option<Region>,

    /// Resources to track.
    #[serde(
        default = "default_zzz_resources",
        deserialize_with = "deserialize_zzz_tracked_resources"
    )]
    pub tracked_resources: Vec<ZzzResourceType>,

    /// Whether to auto-claim daily rewards for this game.
    #[serde(default)]
    pub auto_claim_daily_rewards: bool,

    /// Optional time to auto-claim daily rewards in HH:MM format (UTC+8).
    /// Internally stored as UTC. If not specified, defaults to "00:00" (midnight UTC+8).
    #[serde(default, with = "claim_time_serde")]
    pub auto_claim_time: Option<ClaimTime>,

    /// Per-resource notification settings.
    #[serde(default, deserialize_with = "deserialize_zzz_notifications")]
    pub notifications: HashMap<ZzzResourceType, ResourceNotificationConfig>,
}

fn default_zzz_resources() -> Vec<ZzzResourceType> {
    ZzzResourceType::all().to_vec()
}

/// Wuthering Waves specific configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WuwaConfig {
    /// Whether this game is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Player ID.
    pub player_id: String,

    /// Optional region override.
    pub region: Option<Region>,

    /// Resources to track.
    #[serde(
        default = "default_wuwa_resources",
        deserialize_with = "deserialize_wuwa_tracked_resources"
    )]
    pub tracked_resources: Vec<WuwaResourceType>,

    /// Per-resource notification settings.
    #[serde(default, deserialize_with = "deserialize_wuwa_notifications")]
    pub notifications: HashMap<WuwaResourceType, ResourceNotificationConfig>,
}

fn default_wuwa_resources() -> Vec<WuwaResourceType> {
    WuwaResourceType::all().to_vec()
}

fn parse_resource_key<T: DeserializeOwned>(key: &str) -> Option<T> {
    serde_json::from_value::<T>(serde_json::Value::String(key.to_string())).ok()
}

fn deserialize_tracked_resources<'de, D, T>(
    deserializer: D,
    game_name: &str,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let raw = Vec::<String>::deserialize(deserializer)?;
    let mut resources = Vec::with_capacity(raw.len());

    for key in raw {
        if let Some(resource) = parse_resource_key::<T>(&key) {
            resources.push(resource);
        } else {
            tracing::warn!(
                game = game_name,
                resource_key = %key,
                "Ignoring unknown tracked resource key from config"
            );
        }
    }

    Ok(resources)
}

fn deserialize_notifications<'de, D, T>(
    deserializer: D,
    game_name: &str,
) -> Result<HashMap<T, ResourceNotificationConfig>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Eq + Hash,
{
    let raw = HashMap::<String, ResourceNotificationConfig>::deserialize(deserializer)?;
    let mut notifications = HashMap::with_capacity(raw.len());

    for (key, config) in raw {
        if let Some(resource) = parse_resource_key::<T>(&key) {
            notifications.insert(resource, config);
        } else {
            tracing::warn!(
                game = game_name,
                resource_key = %key,
                "Ignoring unknown notification resource key from config"
            );
        }
    }

    Ok(notifications)
}

fn deserialize_genshin_tracked_resources<'de, D>(
    deserializer: D,
) -> Result<Vec<GenshinResourceType>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_tracked_resources(deserializer, "Genshin Impact")
}

fn deserialize_hsr_tracked_resources<'de, D>(
    deserializer: D,
) -> Result<Vec<HsrResourceType>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_tracked_resources(deserializer, "Honkai: Star Rail")
}

fn deserialize_zzz_tracked_resources<'de, D>(
    deserializer: D,
) -> Result<Vec<ZzzResourceType>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_tracked_resources(deserializer, "Zenless Zone Zero")
}

fn deserialize_wuwa_tracked_resources<'de, D>(
    deserializer: D,
) -> Result<Vec<WuwaResourceType>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_tracked_resources(deserializer, "Wuthering Waves")
}

fn deserialize_genshin_notifications<'de, D>(
    deserializer: D,
) -> Result<HashMap<GenshinResourceType, ResourceNotificationConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_notifications(deserializer, "Genshin Impact")
}

fn deserialize_hsr_notifications<'de, D>(
    deserializer: D,
) -> Result<HashMap<HsrResourceType, ResourceNotificationConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_notifications(deserializer, "Honkai: Star Rail")
}

fn deserialize_zzz_notifications<'de, D>(
    deserializer: D,
) -> Result<HashMap<ZzzResourceType, ResourceNotificationConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_notifications(deserializer, "Zenless Zone Zero")
}

fn deserialize_wuwa_notifications<'de, D>(
    deserializer: D,
) -> Result<HashMap<WuwaResourceType, ResourceNotificationConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_notifications(deserializer, "Wuthering Waves")
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
            .get(&GenshinResourceType::Resin)
            .expect("should have resin config");
        assert!(resin.enabled);
        assert_eq!(resin.notify_minutes_before_full, Some(60));
        assert_eq!(resin.cooldown_minutes, 10);

        let expeditions = config
            .notifications
            .get(&GenshinResourceType::Expeditions)
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

    #[test]
    fn test_unknown_tracked_resource_is_ignored() {
        let toml_str = r#"
            enabled = true
            uid = "123456789"
            tracked_resources = ["resin", "unknown_resource", "expeditions"]
        "#;

        let config: GenshinConfig = toml::from_str(toml_str).expect("should parse config");
        assert_eq!(config.tracked_resources.len(), 2);
        assert!(
            config
                .tracked_resources
                .contains(&GenshinResourceType::Resin)
        );
        assert!(
            config
                .tracked_resources
                .contains(&GenshinResourceType::Expeditions)
        );
    }

    #[test]
    fn test_unknown_notification_resource_is_ignored() {
        let toml_str = r#"
            enabled = true
            uid = "123456789"

            [notifications.resin]
            enabled = true
            cooldown_minutes = 10

            [notifications.unknown_resource]
            enabled = true
            cooldown_minutes = 5
        "#;

        let config: GenshinConfig = toml::from_str(toml_str).expect("should parse config");
        assert_eq!(config.notifications.len(), 1);
        assert!(
            config
                .notifications
                .contains_key(&GenshinResourceType::Resin)
        );
    }
}
