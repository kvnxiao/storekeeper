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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use storekeeper_core::DisplayableResource;

    // =========================================================================
    // DisplayableResource trait tests
    // =========================================================================

    #[test]
    fn test_resin_display_name() {
        let resource = GenshinResource::Resin(StaminaResource::new(120, 160, Local::now(), 480));
        assert_eq!(resource.display_name(), "Original Resin");
    }

    #[test]
    fn test_resin_icon() {
        let resource = GenshinResource::Resin(StaminaResource::new(120, 160, Local::now(), 480));
        assert_eq!(resource.icon(), "resin");
    }

    #[test]
    fn test_parametric_transformer_display_name() {
        let resource = GenshinResource::ParametricTransformer(CooldownResource::ready());
        assert_eq!(resource.display_name(), "Parametric Transformer");
    }

    #[test]
    fn test_parametric_transformer_icon() {
        let resource = GenshinResource::ParametricTransformer(CooldownResource::ready());
        assert_eq!(resource.icon(), "transformer");
    }

    #[test]
    fn test_realm_currency_display_name() {
        let resource =
            GenshinResource::RealmCurrency(StaminaResource::new(1000, 2400, Local::now(), 30));
        assert_eq!(resource.display_name(), "Realm Currency");
    }

    #[test]
    fn test_realm_currency_icon() {
        let resource =
            GenshinResource::RealmCurrency(StaminaResource::new(1000, 2400, Local::now(), 30));
        assert_eq!(resource.icon(), "realm");
    }

    #[test]
    fn test_expeditions_display_name() {
        let resource = GenshinResource::Expeditions(ExpeditionResource::new(3, 5, Local::now()));
        assert_eq!(resource.display_name(), "Expeditions");
    }

    #[test]
    fn test_expeditions_icon() {
        let resource = GenshinResource::Expeditions(ExpeditionResource::new(3, 5, Local::now()));
        assert_eq!(resource.icon(), "expedition");
    }

    // =========================================================================
    // Serde serialization tests (tagged format)
    // =========================================================================

    #[test]
    fn test_resin_serialization_format() {
        let resource = GenshinResource::Resin(StaminaResource::new(120, 160, Local::now(), 480));
        let json = serde_json::to_string(&resource).expect("should serialize");

        // Verify tagged format
        assert!(
            json.contains(r#""type":"resin""#),
            "Should have type tag 'resin', got: {json}"
        );
        assert!(
            json.contains(r#""data":"#),
            "Should have data field, got: {json}"
        );
    }

    #[test]
    fn test_parametric_transformer_serialization_format() {
        let resource = GenshinResource::ParametricTransformer(CooldownResource::ready());
        let json = serde_json::to_string(&resource).expect("should serialize");

        assert!(
            json.contains(r#""type":"parametric_transformer""#),
            "Should have type tag 'parametric_transformer', got: {json}"
        );
    }

    #[test]
    fn test_realm_currency_serialization_format() {
        let resource =
            GenshinResource::RealmCurrency(StaminaResource::new(1000, 2400, Local::now(), 30));
        let json = serde_json::to_string(&resource).expect("should serialize");

        assert!(
            json.contains(r#""type":"realm_currency""#),
            "Should have type tag 'realm_currency', got: {json}"
        );
    }

    #[test]
    fn test_expeditions_serialization_format() {
        let resource = GenshinResource::Expeditions(ExpeditionResource::new(3, 5, Local::now()));
        let json = serde_json::to_string(&resource).expect("should serialize");

        assert!(
            json.contains(r#""type":"expeditions""#),
            "Should have type tag 'expeditions', got: {json}"
        );
    }

    // =========================================================================
    // Serde roundtrip tests
    // =========================================================================

    #[test]
    fn test_resin_serde_roundtrip() {
        let original = GenshinResource::Resin(StaminaResource::new(120, 160, Local::now(), 480));
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: GenshinResource =
            serde_json::from_str(&json).expect("should deserialize");

        let GenshinResource::Resin(resource) = deserialized else {
            unreachable!("Expected Resin variant after deserializing Resin")
        };
        assert_eq!(resource.current, 120);
        assert_eq!(resource.max, 160);
        assert_eq!(resource.regen_rate_seconds, 480);
    }

    #[test]
    fn test_parametric_transformer_serde_roundtrip() {
        let original = GenshinResource::ParametricTransformer(CooldownResource::ready());
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: GenshinResource =
            serde_json::from_str(&json).expect("should deserialize");

        let GenshinResource::ParametricTransformer(resource) = deserialized else {
            unreachable!(
                "Expected ParametricTransformer variant after deserializing ParametricTransformer"
            )
        };
        assert!(resource.is_ready);
    }

    #[test]
    fn test_realm_currency_serde_roundtrip() {
        let original =
            GenshinResource::RealmCurrency(StaminaResource::new(1000, 2400, Local::now(), 30));
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: GenshinResource =
            serde_json::from_str(&json).expect("should deserialize");

        let GenshinResource::RealmCurrency(resource) = deserialized else {
            unreachable!("Expected RealmCurrency variant after deserializing RealmCurrency")
        };
        assert_eq!(resource.current, 1000);
        assert_eq!(resource.max, 2400);
    }

    #[test]
    fn test_expeditions_serde_roundtrip() {
        let original = GenshinResource::Expeditions(ExpeditionResource::new(3, 5, Local::now()));
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: GenshinResource =
            serde_json::from_str(&json).expect("should deserialize");

        let GenshinResource::Expeditions(resource) = deserialized else {
            unreachable!("Expected Expeditions variant after deserializing Expeditions")
        };
        assert_eq!(resource.current_expeditions, 3);
        assert_eq!(resource.max_expeditions, 5);
    }

    // =========================================================================
    // Debug trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_debug() {
        let resource = GenshinResource::Resin(StaminaResource::new(120, 160, Local::now(), 480));
        let debug = format!("{resource:?}");
        assert!(debug.contains("Resin"));
    }

    // =========================================================================
    // Clone trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_clone() {
        let resource = GenshinResource::Resin(StaminaResource::new(120, 160, Local::now(), 480));
        let cloned = resource.clone();

        let GenshinResource::Resin(r) = cloned else {
            unreachable!("Clone should preserve Resin variant")
        };
        assert_eq!(r.current, 120);
        assert_eq!(r.max, 160);
    }
}
