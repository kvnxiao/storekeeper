//! Notification configuration for tracked resources.

use serde::{Deserialize, Serialize};

use super::default_true;

/// Notification configuration for a specific resource.
///
/// Controls when and how often OS notifications are sent for a tracked resource.
/// Supports two threshold modes (mutually exclusive — set one, leave the other `None`):
/// - `notify_minutes_before_full`: fire N minutes before the resource is full
/// - `notify_at_value`: fire when the resource value reaches N (stamina resources only)
///
/// If both are `None`, notifications fire only when the resource is full/ready.
/// If both are `Some` (e.g. manual config edit), `notify_at_value` takes priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNotificationConfig {
    /// Whether notifications are enabled for this resource.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Minutes before full to start notifying. `None` = not using this mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notify_minutes_before_full: Option<u32>,

    /// Resource value at which to notify. `None` = not using this mode.
    /// Only meaningful for stamina-type resources with a current/max value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notify_at_value: Option<u64>,

    /// Minutes between repeated notifications.
    #[serde(default = "default_notification_cooldown")]
    pub cooldown_minutes: u32,
}

fn default_notification_cooldown() -> u32 {
    30
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(config.notify_minutes_before_full, Some(60));
        assert_eq!(config.notify_at_value, None);
        assert_eq!(config.cooldown_minutes, 10);

        let serialized = toml::to_string(&config).expect("should serialize");
        let roundtripped: ResourceNotificationConfig =
            toml::from_str(&serialized).expect("should roundtrip");
        assert_eq!(roundtripped.enabled, config.enabled);
        assert_eq!(
            roundtripped.notify_minutes_before_full,
            config.notify_minutes_before_full
        );
        assert_eq!(roundtripped.notify_at_value, config.notify_at_value);
        assert_eq!(roundtripped.cooldown_minutes, config.cooldown_minutes);
    }

    #[test]
    fn test_resource_notification_config_notify_at_value_roundtrip() {
        let toml_str = r"
            enabled = true
            notify_at_value = 180
            cooldown_minutes = 15
        ";

        let config: ResourceNotificationConfig = toml::from_str(toml_str)
            .expect("should parse notification config with notify_at_value");
        assert!(config.enabled);
        assert_eq!(config.notify_minutes_before_full, None);
        assert_eq!(config.notify_at_value, Some(180));
        assert_eq!(config.cooldown_minutes, 15);

        let serialized = toml::to_string(&config).expect("should serialize");
        assert!(
            serialized.contains("notify_at_value = 180"),
            "serialized should contain notify_at_value, got: {serialized}"
        );
        assert!(
            !serialized.contains("notify_minutes_before_full"),
            "serialized should skip None fields, got: {serialized}"
        );

        let roundtripped: ResourceNotificationConfig =
            toml::from_str(&serialized).expect("should roundtrip");
        assert_eq!(roundtripped.notify_at_value, Some(180));
        assert_eq!(roundtripped.notify_minutes_before_full, None);
    }

    #[test]
    fn test_resource_notification_config_both_none_defaults() {
        let toml_str = r"
            enabled = true
            cooldown_minutes = 30
        ";

        let config: ResourceNotificationConfig =
            toml::from_str(toml_str).expect("should parse config with no threshold");
        assert!(config.enabled);
        assert_eq!(config.notify_minutes_before_full, None);
        assert_eq!(config.notify_at_value, None);
        assert_eq!(config.cooldown_minutes, 30);
    }
}
