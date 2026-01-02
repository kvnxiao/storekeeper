//! Wuthering Waves resource types.

use storekeeper_core::{StaminaResource, game_resource_enum};

game_resource_enum! {
    /// Wuthering Waves resource types.
    pub enum WuwaResource {
        /// Waveplates.
        Waveplates(StaminaResource) => ("Waveplates", "waveplate"),
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
    fn test_waveplates_display_name() {
        let resource = WuwaResource::Waveplates(StaminaResource::new(120, 240, Local::now(), 360));
        assert_eq!(resource.display_name(), "Waveplates");
    }

    #[test]
    fn test_waveplates_icon() {
        let resource = WuwaResource::Waveplates(StaminaResource::new(120, 240, Local::now(), 360));
        assert_eq!(resource.icon(), "waveplate");
    }

    // =========================================================================
    // Serde serialization tests (tagged format)
    // =========================================================================

    #[test]
    fn test_waveplates_serialization_format() {
        let resource = WuwaResource::Waveplates(StaminaResource::new(120, 240, Local::now(), 360));
        let json = serde_json::to_string(&resource).expect("should serialize");

        // Verify tagged format
        assert!(
            json.contains(r#""type":"waveplates""#),
            "Should have type tag 'waveplates', got: {json}"
        );
        assert!(
            json.contains(r#""data":"#),
            "Should have data field, got: {json}"
        );
    }

    // =========================================================================
    // Serde roundtrip tests
    // =========================================================================

    #[test]
    fn test_waveplates_serde_roundtrip() {
        let original = WuwaResource::Waveplates(StaminaResource::new(120, 240, Local::now(), 360));
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: WuwaResource =
            serde_json::from_str(&json).expect("should deserialize");

        let WuwaResource::Waveplates(resource) = deserialized;
        assert_eq!(resource.current, 120);
        assert_eq!(resource.max, 240);
        assert_eq!(resource.regen_rate_seconds, 360);
    }

    // =========================================================================
    // Debug trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_debug() {
        let resource = WuwaResource::Waveplates(StaminaResource::new(120, 240, Local::now(), 360));
        let debug = format!("{resource:?}");
        assert!(debug.contains("Waveplates"));
    }

    // =========================================================================
    // Clone trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_clone() {
        let resource = WuwaResource::Waveplates(StaminaResource::new(120, 240, Local::now(), 360));
        let cloned = resource.clone();

        let WuwaResource::Waveplates(r) = cloned;
        assert_eq!(r.current, 120);
        assert_eq!(r.max, 240);
    }
}
