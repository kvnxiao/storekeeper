//! Zenless Zone Zero resource types.

use storekeeper_core::{StaminaResource, game_resource_enum};

game_resource_enum! {
    /// Zenless Zone Zero resource types.
    pub enum ZzzResource {
        /// Battery charge.
        Battery(StaminaResource) => ("Battery", "battery"),
    }
}
