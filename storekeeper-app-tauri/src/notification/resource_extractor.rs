//! Resource info extraction from JSON data.

use chrono::{DateTime, Utc};
use storekeeper_core::{CooldownResource, ExpeditionResource, StaminaResource};

/// Extracted timing info from a resource JSON object.
pub(crate) struct ResourceInfo {
    /// When the resource will be complete/full/ready.
    pub(crate) completion_at: DateTime<Utc>,
    /// Whether the resource is already complete.
    pub(crate) is_complete: bool,
    /// Current resource value (stamina resources only).
    pub(crate) current: Option<u64>,
    /// Maximum resource value (stamina resources only).
    pub(crate) max: Option<u64>,
    /// Seconds per unit of regeneration (stamina resources only).
    pub(crate) regen_rate_seconds: Option<u64>,
}

/// Extracts completion timing from a resource data object.
///
/// Attempts typed deserialization in order: StaminaResource, CooldownResource,
/// ExpeditionResource. Falls back to `None` if none match.
pub(crate) fn extract_resource_info(data: &serde_json::Value) -> Option<ResourceInfo> {
    if let Ok(stamina) = serde_json::from_value::<StaminaResource>(data.clone()) {
        return Some(ResourceInfo {
            completion_at: stamina.full_at.with_timezone(&Utc),
            is_complete: stamina.is_full(),
            current: Some(u64::from(stamina.current)),
            max: Some(u64::from(stamina.max)),
            regen_rate_seconds: Some(u64::from(stamina.regen_rate_seconds)),
        });
    }

    if let Ok(cooldown) = serde_json::from_value::<CooldownResource>(data.clone()) {
        return Some(ResourceInfo {
            completion_at: cooldown.ready_at.with_timezone(&Utc),
            is_complete: cooldown.is_ready,
            current: None,
            max: None,
            regen_rate_seconds: None,
        });
    }

    if let Ok(expedition) = serde_json::from_value::<ExpeditionResource>(data.clone()) {
        let completion_at = expedition.earliest_finish_at.with_timezone(&Utc);
        return Some(ResourceInfo {
            completion_at,
            is_complete: completion_at <= Utc::now(),
            current: None,
            max: None,
            regen_rate_seconds: None,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;

    #[test]
    fn test_extract_stamina_resource() {
        let future = Utc::now() + TimeDelta::hours(2);
        let data = serde_json::json!({
            "current": 100,
            "max": 160,
            "fullAt": future.to_rfc3339(),
            "regenRateSeconds": 480
        });

        let info = extract_resource_info(&data).expect("should extract stamina resource");
        assert!(!info.is_complete);
        assert!((info.completion_at - future).num_seconds().abs() < 2);
    }

    #[test]
    fn test_extract_stamina_resource_full() {
        let past = Utc::now() - TimeDelta::hours(1);
        let data = serde_json::json!({
            "current": 160,
            "max": 160,
            "fullAt": past.to_rfc3339(),
            "regenRateSeconds": 480
        });

        let info = extract_resource_info(&data).expect("should extract full stamina resource");
        assert!(info.is_complete);
    }

    #[test]
    fn test_extract_cooldown_resource_ready() {
        let past = Utc::now() - TimeDelta::hours(1);
        let data = serde_json::json!({
            "isReady": true,
            "readyAt": past.to_rfc3339()
        });

        let info = extract_resource_info(&data).expect("should extract cooldown resource");
        assert!(info.is_complete);
    }

    #[test]
    fn test_extract_cooldown_resource_not_ready() {
        let future = Utc::now() + TimeDelta::hours(12);
        let data = serde_json::json!({
            "isReady": false,
            "readyAt": future.to_rfc3339()
        });

        let info = extract_resource_info(&data).expect("should extract cooldown resource");
        assert!(!info.is_complete);
    }

    #[test]
    fn test_extract_expedition_resource_completed() {
        let past = Utc::now() - TimeDelta::minutes(30);
        let data = serde_json::json!({
            "currentExpeditions": 3,
            "maxExpeditions": 5,
            "earliestFinishAt": past.to_rfc3339()
        });

        let info = extract_resource_info(&data).expect("should extract expedition resource");
        assert!(info.is_complete);
    }

    #[test]
    fn test_extract_expedition_resource_pending() {
        let future = Utc::now() + TimeDelta::hours(6);
        let data = serde_json::json!({
            "currentExpeditions": 3,
            "maxExpeditions": 5,
            "earliestFinishAt": future.to_rfc3339()
        });

        let info = extract_resource_info(&data).expect("should extract expedition resource");
        assert!(!info.is_complete);
    }

    #[test]
    fn test_extract_unknown_resource_returns_none() {
        let data = serde_json::json!({
            "someUnknownField": 42
        });

        assert!(extract_resource_info(&data).is_none());
    }
}
