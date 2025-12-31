//! Resource types representing in-game stamina and cooldown resources.

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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StaminaResource {
    /// Current amount of the resource.
    pub current: u32,
    /// Maximum capacity of the resource.
    pub max: u32,
    /// Seconds until the resource is fully recovered (None if already full).
    pub seconds_until_full: Option<u64>,
    /// How many seconds it takes to regenerate one unit.
    pub regen_rate_seconds: u32,
}

impl StaminaResource {
    /// Creates a new stamina resource.
    #[must_use = "this returns a new StaminaResource"]
    pub fn new(
        current: u32,
        max: u32,
        seconds_until_full: Option<u64>,
        regen_rate_seconds: u32,
    ) -> Self {
        Self {
            current,
            max,
            seconds_until_full,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CooldownResource {
    /// Whether the item is ready to use.
    pub is_ready: bool,
    /// Seconds until the item is ready (None if already ready).
    pub seconds_until_ready: Option<u64>,
}

impl CooldownResource {
    /// Creates a new cooldown resource.
    #[must_use = "this returns a new CooldownResource"]
    pub fn new(is_ready: bool, seconds_until_ready: Option<u64>) -> Self {
        Self {
            is_ready,
            seconds_until_ready,
        }
    }

    /// Creates a cooldown resource that is ready.
    #[must_use = "this returns a new CooldownResource"]
    pub fn ready() -> Self {
        Self {
            is_ready: true,
            seconds_until_ready: None,
        }
    }

    /// Creates a cooldown resource that is on cooldown.
    #[must_use = "this returns a new CooldownResource"]
    pub fn on_cooldown(seconds_until_ready: u64) -> Self {
        Self {
            is_ready: false,
            seconds_until_ready: Some(seconds_until_ready),
        }
    }
}

/// Expedition tracking data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpeditionResource {
    /// Number of currently active expeditions.
    pub current_expeditions: u32,
    /// Maximum number of expeditions allowed.
    pub max_expeditions: u32,
    /// Seconds until the earliest expedition finishes (None if no active expeditions).
    pub earliest_finish_seconds: Option<u64>,
}

impl ExpeditionResource {
    /// Creates a new expedition resource.
    #[must_use = "this returns a new ExpeditionResource"]
    pub fn new(
        current_expeditions: u32,
        max_expeditions: u32,
        earliest_finish_seconds: Option<u64>,
    ) -> Self {
        Self {
            current_expeditions,
            max_expeditions,
            earliest_finish_seconds,
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
        self.earliest_finish_seconds == Some(0)
    }
}
