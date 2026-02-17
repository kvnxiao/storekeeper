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

impl ResourceInfo {
    /// Estimates the current resource value from `completion_at` and regen rate.
    ///
    /// The cached `current` field can be stale (set at API-fetch time), so this
    /// computes the value from elapsed time instead.  Falls back to the cached
    /// `current` when max/rate are unavailable.
    pub(crate) fn estimated_current(&self, now: DateTime<Utc>) -> Option<u64> {
        let (max, rate) = match (self.max, self.regen_rate_seconds) {
            (Some(m), Some(r)) if r > 0 => (m, r),
            _ => return self.current,
        };

        if self.is_complete || now >= self.completion_at {
            return Some(max);
        }

        let secs_to_full = (self.completion_at - now).num_seconds();
        if secs_to_full <= 0 {
            return Some(max);
        }

        // Ceiling division: partial progress toward the next unit hasn't ticked yet.
        // Safety: secs_to_full is guaranteed positive by the check above.
        let secs = u64::try_from(secs_to_full).unwrap_or(0);
        let remaining_units = secs.div_ceil(rate);
        Some(max.saturating_sub(remaining_units))
    }
}

/// Extracts completion timing from a resource data object.
///
/// Checks discriminating fields before attempting deserialization to minimize
/// `serde_json::Value` clones (which are deep copies).
pub(crate) fn extract_resource_info(data: &serde_json::Value) -> Option<ResourceInfo> {
    let obj = data.as_object()?;

    if obj.contains_key("regenRateSeconds") {
        let stamina: StaminaResource = serde_json::from_value(data.clone()).ok()?;
        return Some(ResourceInfo {
            completion_at: stamina.full_at.with_timezone(&Utc),
            is_complete: stamina.is_full(),
            current: Some(u64::from(stamina.current)),
            max: Some(u64::from(stamina.max)),
            regen_rate_seconds: Some(u64::from(stamina.regen_rate_seconds)),
        });
    }

    if obj.contains_key("isReady") {
        let cooldown: CooldownResource = serde_json::from_value(data.clone()).ok()?;
        return Some(ResourceInfo {
            completion_at: cooldown.ready_at.with_timezone(&Utc),
            is_complete: cooldown.is_ready,
            current: None,
            max: None,
            regen_rate_seconds: None,
        });
    }

    if obj.contains_key("earliestFinishAt") {
        let expedition: ExpeditionResource = serde_json::from_value(data.clone()).ok()?;
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

    // =========================================================================
    // estimated_current tests
    // =========================================================================

    fn stamina_info(
        completion_at: DateTime<Utc>,
        is_complete: bool,
        current: u64,
        max: u64,
        rate: u64,
    ) -> ResourceInfo {
        ResourceInfo {
            completion_at,
            is_complete,
            current: Some(current),
            max: Some(max),
            regen_rate_seconds: Some(rate),
        }
    }

    #[test]
    fn test_estimated_current_matches_threshold_exactly() {
        // WuWa: max=240, rate=360s. At exactly 360 min to full → current=180.
        let now = Utc::now();
        let info = stamina_info(now + TimeDelta::minutes(360), false, 179, 240, 360);
        assert_eq!(info.estimated_current(now), Some(180));
    }

    #[test]
    fn test_estimated_current_one_second_before_tick() {
        // 1 second before the 180th unit ticks: still 179.
        let now = Utc::now();
        let info = stamina_info(
            now + TimeDelta::minutes(360) + TimeDelta::seconds(1),
            false,
            179,
            240,
            360,
        );
        assert_eq!(info.estimated_current(now), Some(179));
    }

    #[test]
    fn test_estimated_current_one_second_after_tick() {
        // 1 second after the 180th unit ticked: 180.
        let now = Utc::now();
        let info = stamina_info(
            now + TimeDelta::minutes(360) - TimeDelta::seconds(1),
            false,
            179,
            240,
            360,
        );
        assert_eq!(info.estimated_current(now), Some(180));
    }

    #[test]
    fn test_estimated_current_when_full() {
        let now = Utc::now();
        let info = stamina_info(now - TimeDelta::minutes(5), true, 240, 240, 360);
        assert_eq!(info.estimated_current(now), Some(240));
    }

    #[test]
    fn test_estimated_current_past_completion() {
        // completion_at is in the past but is_complete not set (stale flag).
        let now = Utc::now();
        let info = stamina_info(now - TimeDelta::minutes(1), false, 239, 240, 360);
        assert_eq!(info.estimated_current(now), Some(240));
    }

    #[test]
    fn test_estimated_current_no_rate_falls_back() {
        let now = Utc::now();
        let info = ResourceInfo {
            completion_at: now + TimeDelta::hours(1),
            is_complete: false,
            current: Some(100),
            max: Some(160),
            regen_rate_seconds: None,
        };
        assert_eq!(info.estimated_current(now), Some(100));
    }
}
