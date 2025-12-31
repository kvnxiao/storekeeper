//! Honkai: Star Rail resource types.

use storekeeper_core::{StaminaResource, game_resource_enum};

game_resource_enum! {
    /// Honkai: Star Rail resource types.
    pub enum HsrResource {
        /// Trailblaze Power.
        TrailblazePower(StaminaResource) => ("Trailblaze Power", "power"),
    }
}
