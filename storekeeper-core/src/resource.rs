//! Resource types representing in-game stamina and cooldown resources.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

/// Trait for game resources that can be displayed in the UI.
///
/// This trait provides common methods for UI rendering across all game resource types.
pub trait DisplayableResource {
    /// Returns the human-readable display name for this resource.
    fn display_name(&self) -> &'static str;

    /// Returns the icon identifier for this resource.
    fn icon(&self) -> &'static str;
}

/// Shared resource data for stamina-like resources.
///
/// All stamina resources (Resin, Trailblaze Power, Battery, Waveplates)
/// share these common fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaminaResource {
    /// Current amount of the resource.
    pub current: u32,
    /// Maximum capacity of the resource.
    pub max: u32,
    /// Instant when the resource will be fully recovered.
    pub full_at: Timestamp,
    /// How many seconds it takes to regenerate one unit.
    pub regen_rate_seconds: u32,
}

impl StaminaResource {
    /// Creates a new stamina resource.
    #[must_use = "this returns a new StaminaResource"]
    pub fn new(current: u32, max: u32, full_at: Timestamp, regen_rate_seconds: u32) -> Self {
        Self {
            current,
            max,
            full_at,
            regen_rate_seconds,
        }
    }

    /// Returns true if the resource is at maximum capacity.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Returns the percentage of the resource that is filled (0.0 to 1.0).
    #[must_use]
    pub fn fill_percentage(&self) -> f64 {
        if self.max == 0 {
            return 0.0;
        }
        f64::from(self.current) / f64::from(self.max)
    }
}

/// Shared cooldown data for items like Parametric Transformer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CooldownResource {
    /// Whether the item is ready to use.
    pub is_ready: bool,
    /// Instant when the item will be ready.
    pub ready_at: Timestamp,
}

impl CooldownResource {
    /// Creates a new cooldown resource.
    #[must_use = "this returns a new CooldownResource"]
    pub fn new(is_ready: bool, ready_at: Timestamp) -> Self {
        Self { is_ready, ready_at }
    }

    /// Creates a cooldown resource that is ready.
    #[must_use = "this returns a new CooldownResource"]
    pub fn ready() -> Self {
        Self {
            is_ready: true,
            ready_at: Timestamp::now(),
        }
    }

    /// Creates a cooldown resource that is on cooldown.
    #[must_use = "this returns a new CooldownResource"]
    pub fn on_cooldown(ready_at: Timestamp) -> Self {
        Self {
            is_ready: false,
            ready_at,
        }
    }
}

/// Expedition tracking data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpeditionResource {
    /// Number of currently active expeditions.
    pub current_expeditions: u32,
    /// Maximum number of expeditions allowed.
    pub max_expeditions: u32,
    /// Instant when the earliest expedition finishes.
    pub earliest_finish_at: Timestamp,
}

impl ExpeditionResource {
    /// Creates a new expedition resource.
    #[must_use = "this returns a new ExpeditionResource"]
    pub fn new(
        current_expeditions: u32,
        max_expeditions: u32,
        earliest_finish_at: Timestamp,
    ) -> Self {
        Self {
            current_expeditions,
            max_expeditions,
            earliest_finish_at,
        }
    }

    /// Returns true if all expedition slots are in use.
    #[must_use]
    pub fn all_slots_used(&self) -> bool {
        self.current_expeditions >= self.max_expeditions
    }

    /// Returns true if any expedition is ready to collect.
    #[must_use]
    pub fn has_completed(&self) -> bool {
        self.earliest_finish_at <= Timestamp::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::SignedDuration;

    // =========================================================================
    // StaminaResource tests
    // =========================================================================

    #[test]
    fn test_stamina_resource_new() {
        let now = Timestamp::now();
        let resource = StaminaResource::new(100, 160, now, 480);

        assert_eq!(resource.current, 100);
        assert_eq!(resource.max, 160);
        assert_eq!(resource.full_at, now);
        assert_eq!(resource.regen_rate_seconds, 480);
    }

    #[test]
    fn test_stamina_is_full_when_current_equals_max() {
        let resource = StaminaResource::new(160, 160, Timestamp::now(), 480);
        assert!(resource.is_full(), "Should be full when current equals max");
    }

    #[test]
    fn test_stamina_is_full_when_current_exceeds_max() {
        // Some games allow overflow (e.g., from fragile resin)
        let resource = StaminaResource::new(180, 160, Timestamp::now(), 480);
        assert!(
            resource.is_full(),
            "Should be full when current exceeds max"
        );
    }

    #[test]
    fn test_stamina_is_not_full_when_current_less_than_max() {
        let resource = StaminaResource::new(100, 160, Timestamp::now(), 480);
        assert!(
            !resource.is_full(),
            "Should not be full when current is less than max"
        );
    }

    #[test]
    fn test_stamina_is_not_full_at_zero() {
        let resource = StaminaResource::new(0, 160, Timestamp::now(), 480);
        assert!(!resource.is_full(), "Should not be full at zero");
    }

    #[test]
    fn test_fill_percentage_at_zero() {
        let resource = StaminaResource::new(0, 160, Timestamp::now(), 480);
        assert!(
            (resource.fill_percentage() - 0.0).abs() < f64::EPSILON,
            "Fill percentage should be 0.0 at zero"
        );
    }

    #[test]
    fn test_fill_percentage_at_half() {
        let resource = StaminaResource::new(80, 160, Timestamp::now(), 480);
        assert!(
            (resource.fill_percentage() - 0.5).abs() < f64::EPSILON,
            "Fill percentage should be 0.5 at half"
        );
    }

    #[test]
    fn test_fill_percentage_at_full() {
        let resource = StaminaResource::new(160, 160, Timestamp::now(), 480);
        assert!(
            (resource.fill_percentage() - 1.0).abs() < f64::EPSILON,
            "Fill percentage should be 1.0 at full"
        );
    }

    #[test]
    fn test_fill_percentage_over_max() {
        let resource = StaminaResource::new(200, 160, Timestamp::now(), 480);
        assert!(
            resource.fill_percentage() > 1.0,
            "Fill percentage should be > 1.0 when over max"
        );
        assert!(
            (resource.fill_percentage() - 1.25).abs() < f64::EPSILON,
            "Fill percentage should be 1.25 (200/160)"
        );
    }

    #[test]
    fn test_fill_percentage_with_max_zero() {
        // Edge case: max is zero (should return 0.0, not divide by zero)
        let resource = StaminaResource::new(0, 0, Timestamp::now(), 480);
        assert!(
            (resource.fill_percentage() - 0.0).abs() < f64::EPSILON,
            "Fill percentage should be 0.0 when max is zero"
        );
    }

    #[test]
    fn test_fill_percentage_with_current_nonzero_max_zero() {
        // Edge case: current > 0 but max is 0
        let resource = StaminaResource::new(100, 0, Timestamp::now(), 480);
        assert!(
            (resource.fill_percentage() - 0.0).abs() < f64::EPSILON,
            "Fill percentage should be 0.0 when max is zero (even with current > 0)"
        );
    }

    // =========================================================================
    // CooldownResource tests
    // =========================================================================

    #[test]
    fn test_cooldown_resource_new() {
        let now = Timestamp::now();
        let resource = CooldownResource::new(true, now);

        assert!(resource.is_ready);
        assert_eq!(resource.ready_at, now);
    }

    #[test]
    fn test_cooldown_ready_factory() {
        let before = Timestamp::now();
        let resource = CooldownResource::ready();
        let after = Timestamp::now();

        assert!(resource.is_ready, "ready() should create a ready resource");
        assert!(
            resource.ready_at >= before && resource.ready_at <= after,
            "ready_at should be approximately now"
        );
    }

    #[test]
    fn test_cooldown_on_cooldown_factory() {
        let future_time = Timestamp::now() + SignedDuration::from_hours(24);
        let resource = CooldownResource::on_cooldown(future_time);

        assert!(
            !resource.is_ready,
            "on_cooldown() should create a not-ready resource"
        );
        assert_eq!(resource.ready_at, future_time);
    }

    // =========================================================================
    // ExpeditionResource tests
    // =========================================================================

    #[test]
    fn test_expedition_resource_new() {
        let now = Timestamp::now();
        let resource = ExpeditionResource::new(3, 5, now);

        assert_eq!(resource.current_expeditions, 3);
        assert_eq!(resource.max_expeditions, 5);
        assert_eq!(resource.earliest_finish_at, now);
    }

    #[test]
    fn test_all_slots_used_when_full() {
        let resource = ExpeditionResource::new(5, 5, Timestamp::now());
        assert!(
            resource.all_slots_used(),
            "Should be full when current equals max"
        );
    }

    #[test]
    fn test_all_slots_used_when_over_max() {
        // Edge case: current > max (shouldn't happen in practice)
        let resource = ExpeditionResource::new(6, 5, Timestamp::now());
        assert!(
            resource.all_slots_used(),
            "Should be full when current exceeds max"
        );
    }

    #[test]
    fn test_all_slots_not_used_when_below_max() {
        let resource = ExpeditionResource::new(3, 5, Timestamp::now());
        assert!(
            !resource.all_slots_used(),
            "Should not be full when current < max"
        );
    }

    #[test]
    fn test_all_slots_not_used_at_zero() {
        let resource = ExpeditionResource::new(0, 5, Timestamp::now());
        assert!(!resource.all_slots_used(), "Should not be full at zero");
    }

    #[test]
    fn test_has_completed_when_finish_time_is_past() {
        let past_time = Timestamp::now() - SignedDuration::from_hours(1);
        let resource = ExpeditionResource::new(3, 5, past_time);
        assert!(
            resource.has_completed(),
            "Should have completed when finish time is in the past"
        );
    }

    #[test]
    fn test_has_completed_when_finish_time_is_now() {
        // Note: This test is slightly flaky due to timing, but the logic is correct
        let now = Timestamp::now();
        let resource = ExpeditionResource::new(3, 5, now);
        // The check is earliest_finish_at <= Timestamp::now(), so it should be completed
        assert!(
            resource.has_completed(),
            "Should have completed when finish time is now"
        );
    }

    #[test]
    fn test_has_not_completed_when_finish_time_is_future() {
        let future_time = Timestamp::now() + SignedDuration::from_hours(1);
        let resource = ExpeditionResource::new(3, 5, future_time);
        assert!(
            !resource.has_completed(),
            "Should not have completed when finish time is in the future"
        );
    }

    // =========================================================================
    // Serde tests
    // =========================================================================

    #[test]
    fn test_stamina_resource_serde_roundtrip() {
        let now = Timestamp::now();
        let resource = StaminaResource::new(100, 160, now, 480);

        let json = serde_json::to_string(&resource).expect("should serialize");
        let deserialized: StaminaResource =
            serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(resource.current, deserialized.current);
        assert_eq!(resource.max, deserialized.max);
        assert_eq!(resource.regen_rate_seconds, deserialized.regen_rate_seconds);
        // jiff::Timestamp roundtrips losslessly through RFC3339, so the value is exact.
        assert_eq!(
            resource.full_at, deserialized.full_at,
            "full_at should roundtrip exactly"
        );
    }

    #[test]
    fn test_stamina_resource_serializes_full_at_as_utc_z() {
        // Locks the JSON contract consumed by the frontend: the `fullAt` field
        // is an RFC3339 string in UTC with a trailing `Z`.
        let ts = Timestamp::from_second(1_704_067_200).expect("valid timestamp");
        let resource = StaminaResource::new(100, 160, ts, 480);
        let value = serde_json::to_value(&resource).expect("should serialize");
        assert_eq!(value["fullAt"], "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_cooldown_resource_serde_roundtrip() {
        let resource = CooldownResource::ready();

        let json = serde_json::to_string(&resource).expect("should serialize");
        let deserialized: CooldownResource =
            serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(resource.is_ready, deserialized.is_ready);
    }

    #[test]
    fn test_expedition_resource_serde_roundtrip() {
        let now = Timestamp::now();
        let resource = ExpeditionResource::new(3, 5, now);

        let json = serde_json::to_string(&resource).expect("should serialize");
        let deserialized: ExpeditionResource =
            serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(
            resource.current_expeditions,
            deserialized.current_expeditions
        );
        assert_eq!(resource.max_expeditions, deserialized.max_expeditions);
    }
}
