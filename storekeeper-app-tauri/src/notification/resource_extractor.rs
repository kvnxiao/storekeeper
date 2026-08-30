//! Resource info extraction from JSON data.

use jiff::Timestamp;
use storekeeper_core::CooldownResource;
use storekeeper_core::ExpeditionResource;
use storekeeper_core::StaminaResource;

/// Extracted timing info from a resource JSON object.
pub(crate) struct ResourceInfo {
    /// When the resource will be complete/full/ready.
    pub(crate) completion_at: Timestamp,
    /// Whether the resource is already complete.
    pub(crate) is_complete: bool,
    /// Current resource value (stamina resources only).
    pub(crate) current: Option<u64>,
    /// Maximum resource value (stamina resources only).
    pub(crate) max: Option<u64>,
    /// Seconds between accrual steps (stamina resources only).
    pub(crate) regen_rate_seconds: Option<u64>,
    /// Units credited per accrual step (stamina resources only).
    pub(crate) regen_step_units: Option<u64>,
}

impl ResourceInfo {
    /// Estimates the current resource value from `completion_at` and the
    /// accrual step.
    ///
    /// Uses elapsed time instead of the cached `current` field, which is set at
    /// API-fetch time. Falls back to `current` when max or rate is unavailable.
    /// While an incomplete resource is recovering, clamps the estimate to
    /// `current` when `max - current` is not divisible by the step size.
    pub(crate) fn estimated_current(&self, now: Timestamp) -> Option<u64> {
        let (max, step_seconds) = match (self.max, self.regen_rate_seconds) {
            (Some(m), Some(r)) if r > 0 => (m, r),
            _ => return self.current,
        };

        if self.is_complete || now >= self.completion_at {
            return Some(max);
        }

        let secs_to_full = self.completion_at.duration_since(now).as_secs();
        if secs_to_full <= 0 {
            return Some(max);
        }

        let secs = u64::try_from(secs_to_full).unwrap_or(0);
        let step_units = self.regen_step_units.unwrap_or(1);
        let remaining_steps = secs.div_ceil(step_seconds);
        let remaining_units = remaining_steps.saturating_mul(step_units);
        let estimated = max.saturating_sub(remaining_units);
        Some(
            self.current
                .map_or(estimated, |cached| estimated.max(cached)),
        )
    }
}

/// Extracts completion timing from a resource data object.
///
/// Uses `resource_type` to deserialize into exactly one expected shape.
pub(crate) fn extract_resource_info(
    resource_type: &str,
    data: &serde_json::Value,
) -> Option<ResourceInfo> {
    match resource_type {
        "parametric_transformer" => serde_json::from_value::<CooldownResource>(data.clone())
            .ok()
            .map(|cooldown| ResourceInfo {
                completion_at: cooldown.ready_at,
                is_complete: cooldown.is_ready,
                current: None,
                max: None,
                regen_rate_seconds: None,
                regen_step_units: None,
            }),
        "expeditions" => serde_json::from_value::<ExpeditionResource>(data.clone())
            .ok()
            .map(|expedition| {
                let completion_at = expedition.earliest_finish_at;
                ResourceInfo {
                    completion_at,
                    is_complete: completion_at <= Timestamp::now(),
                    current: None,
                    max: None,
                    regen_rate_seconds: None,
                    regen_step_units: None,
                }
            }),
        _ => serde_json::from_value::<StaminaResource>(data.clone())
            .ok()
            .map(|stamina| ResourceInfo {
                completion_at: stamina.full_at,
                is_complete: stamina.is_full(),
                current: Some(u64::from(stamina.current)),
                max: Some(u64::from(stamina.max)),
                regen_rate_seconds: Some(u64::from(stamina.regen_rate_seconds)),
                regen_step_units: Some(u64::from(stamina.regen_step_units)),
            }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::SignedDuration;

    #[test]
    fn extract_stamina_resource() {
        let future = Timestamp::now() + SignedDuration::from_hours(2);
        let data = serde_json::json!({
            "current": 100,
            "max": 160,
            "fullAt": future.to_string(),
            "regenRateSeconds": 480,
            "regenStepUnits": 1
        });

        let info = extract_resource_info("resin", &data).expect("should extract stamina resource");
        assert!(!info.is_complete);
        assert!(info.completion_at.duration_since(future).as_secs().abs() < 2);
    }

    #[test]
    fn extract_stamina_resource_full() {
        let past = Timestamp::now() - SignedDuration::from_hours(1);
        let data = serde_json::json!({
            "current": 160,
            "max": 160,
            "fullAt": past.to_string(),
            "regenRateSeconds": 480,
            "regenStepUnits": 1
        });

        let info =
            extract_resource_info("resin", &data).expect("should extract full stamina resource");
        assert!(info.is_complete);
    }

    #[test]
    fn extract_cooldown_resource_ready() {
        let past = Timestamp::now() - SignedDuration::from_hours(1);
        let data = serde_json::json!({
            "isReady": true,
            "readyAt": past.to_string()
        });

        let info = extract_resource_info("parametric_transformer", &data)
            .expect("should extract cooldown resource");
        assert!(info.is_complete);
    }

    #[test]
    fn extract_cooldown_resource_not_ready() {
        let future = Timestamp::now() + SignedDuration::from_hours(12);
        let data = serde_json::json!({
            "isReady": false,
            "readyAt": future.to_string()
        });

        let info = extract_resource_info("parametric_transformer", &data)
            .expect("should extract cooldown resource");
        assert!(!info.is_complete);
    }

    #[test]
    fn extract_expedition_resource_completed() {
        let past = Timestamp::now() - SignedDuration::from_mins(30);
        let data = serde_json::json!({
            "currentExpeditions": 3,
            "maxExpeditions": 5,
            "earliestFinishAt": past.to_string()
        });

        let info = extract_resource_info("expeditions", &data)
            .expect("should extract expedition resource");
        assert!(info.is_complete);
    }

    #[test]
    fn extract_expedition_resource_pending() {
        let future = Timestamp::now() + SignedDuration::from_hours(6);
        let data = serde_json::json!({
            "currentExpeditions": 3,
            "maxExpeditions": 5,
            "earliestFinishAt": future.to_string()
        });

        let info = extract_resource_info("expeditions", &data)
            .expect("should extract expedition resource");
        assert!(!info.is_complete);
    }

    #[test]
    fn extract_unknown_resource_returns_none() {
        let data = serde_json::json!({
            "someUnknownField": 42
        });

        assert!(extract_resource_info("unknown_resource", &data).is_none());
    }

    fn stamina_info(
        completion_at: Timestamp,
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
            regen_step_units: Some(1),
        }
    }

    #[test]
    fn estimated_current_matches_threshold_exactly() {
        // WuWa: max=240, rate=360s. At exactly 360 min to full → current=180.
        let now = Timestamp::now();
        let info = stamina_info(now + SignedDuration::from_mins(360), false, 179, 240, 360);
        assert_eq!(info.estimated_current(now), Some(180));
    }

    #[test]
    fn estimated_current_one_second_before_tick() {
        // 1 second before the 180th unit ticks: still 179.
        let now = Timestamp::now();
        let info = stamina_info(
            now + SignedDuration::from_mins(360) + SignedDuration::from_secs(1),
            false,
            179,
            240,
            360,
        );
        assert_eq!(info.estimated_current(now), Some(179));
    }

    #[test]
    fn estimated_current_one_second_after_tick() {
        // 1 second after the 180th unit ticked: 180.
        let now = Timestamp::now();
        let info = stamina_info(
            now + SignedDuration::from_mins(360) - SignedDuration::from_secs(1),
            false,
            179,
            240,
            360,
        );
        assert_eq!(info.estimated_current(now), Some(180));
    }

    #[test]
    fn estimated_current_when_full() {
        let now = Timestamp::now();
        let info = stamina_info(now - SignedDuration::from_mins(5), true, 240, 240, 360);
        assert_eq!(info.estimated_current(now), Some(240));
    }

    #[test]
    fn estimated_current_past_completion() {
        // completion_at is in the past but is_complete not set (stale flag).
        let now = Timestamp::now();
        let info = stamina_info(now - SignedDuration::from_mins(1), false, 239, 240, 360);
        assert_eq!(info.estimated_current(now), Some(240));
    }

    fn realm_info(now: Timestamp, secs_to_full: i64) -> ResourceInfo {
        ResourceInfo {
            completion_at: now + SignedDuration::from_secs(secs_to_full),
            is_complete: false,
            current: Some(1980),
            max: Some(2400),
            regen_rate_seconds: Some(3600),
            regen_step_units: Some(30),
        }
    }

    #[test]
    fn estimated_current_holds_flat_between_hourly_credits() {
        let now = Timestamp::now();
        assert_eq!(
            realm_info(now, 14 * 3600).estimated_current(now),
            Some(1980)
        );
        assert_eq!(
            realm_info(now, 13 * 3600 + 1).estimated_current(now),
            Some(1980)
        );
    }

    #[test]
    fn estimated_current_never_reads_below_the_polled_value() {
        let now = Timestamp::now();
        let info = ResourceInfo {
            completion_at: now + SignedDuration::from_secs(47 * 3600),
            is_complete: false,
            current: Some(1000),
            max: Some(2400),
            regen_rate_seconds: Some(3600),
            regen_step_units: Some(30),
        };
        assert_eq!(info.estimated_current(now), Some(1000));
    }

    #[test]
    fn estimated_current_advances_one_step_per_hour() {
        let now = Timestamp::now();
        assert_eq!(
            realm_info(now, 13 * 3600).estimated_current(now),
            Some(2010)
        );
    }

    #[test]
    fn estimated_current_no_rate_falls_back() {
        let now = Timestamp::now();
        let info = ResourceInfo {
            completion_at: now + SignedDuration::from_hours(1),
            is_complete: false,
            current: Some(100),
            max: Some(160),
            regen_rate_seconds: None,
            regen_step_units: None,
        };
        assert_eq!(info.estimated_current(now), Some(100));
    }
}
