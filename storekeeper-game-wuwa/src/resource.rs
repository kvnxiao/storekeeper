//! Wuthering Waves resource types.

use storekeeper_core::{StaminaResource, game_resource_enum};

game_resource_enum! {
    /// Wuthering Waves resource types.
    pub enum WuwaResource {
        /// Waveplates.
        Waveplates(StaminaResource) => ("Waveplates", "waveplate"),
    }
}
