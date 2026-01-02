//! Honkai: Star Rail resource types.

use storekeeper_core::{StaminaResource, game_resource_enum};

game_resource_enum! {
    /// Honkai: Star Rail resource types.
    pub enum HsrResource {
        /// Trailblaze Power.
        TrailblazePower(StaminaResource) => ("Trailblaze Power", "power"),
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
    fn test_trailblaze_power_display_name() {
        let resource =
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Local::now(), 360));
        assert_eq!(resource.display_name(), "Trailblaze Power");
    }

    #[test]
    fn test_trailblaze_power_icon() {
        let resource =
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Local::now(), 360));
        assert_eq!(resource.icon(), "power");
    }

    // =========================================================================
    // Serde serialization tests (tagged format)
    // =========================================================================

    #[test]
    fn test_trailblaze_power_serialization_format() {
        let resource =
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Local::now(), 360));
        let json = serde_json::to_string(&resource).expect("should serialize");

        // Verify tagged format
        assert!(
            json.contains(r#""type":"trailblaze_power""#),
            "Should have type tag 'trailblaze_power', got: {json}"
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
    fn test_trailblaze_power_serde_roundtrip() {
        let original =
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Local::now(), 360));
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: HsrResource = serde_json::from_str(&json).expect("should deserialize");

        let HsrResource::TrailblazePower(resource) = deserialized;
        assert_eq!(resource.current, 180);
        assert_eq!(resource.max, 240);
        assert_eq!(resource.regen_rate_seconds, 360);
    }

    // =========================================================================
    // Debug trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_debug() {
        let resource =
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Local::now(), 360));
        let debug = format!("{resource:?}");
        assert!(debug.contains("TrailblazePower"));
    }

    // =========================================================================
    // Clone trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_clone() {
        let resource =
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Local::now(), 360));
        let cloned = resource.clone();

        let HsrResource::TrailblazePower(r) = cloned;
        assert_eq!(r.current, 180);
        assert_eq!(r.max, 240);
    }
}
