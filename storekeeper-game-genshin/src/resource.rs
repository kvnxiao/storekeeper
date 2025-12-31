//! Genshin Impact resource types.

use storekeeper_core::{CooldownResource, ExpeditionResource, StaminaResource, game_resource_enum};

game_resource_enum! {
    /// Genshin Impact resource types.
    pub enum GenshinResource {
        /// Original Resin.
        Resin(StaminaResource) => ("Original Resin", "resin"),
        /// Parametric Transformer cooldown.
        ParametricTransformer(CooldownResource) => ("Parametric Transformer", "transformer"),
        /// Serenitea Pot Realm Currency.
        RealmCurrency(StaminaResource) => ("Realm Currency", "realm"),
        /// Expedition tracking.
        Expeditions(ExpeditionResource) => ("Expeditions", "expedition"),
    }
}
