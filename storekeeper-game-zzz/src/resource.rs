//! Zenless Zone Zero resource types.

use storekeeper_core::{StaminaResource, game_resource_enum};

game_resource_enum! {
    /// Zenless Zone Zero resource types.
    pub enum ZzzResource {
        /// Battery charge.
        Battery(StaminaResource) => ("Battery", "battery"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::Timestamp;
    use storekeeper_core::DisplayableResource;

    // =========================================================================
    // DisplayableResource trait tests
    // =========================================================================

    #[test]
    fn test_battery_display_name() {
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        assert_eq!(resource.display_name(), "Battery");
    }

    #[test]
    fn test_battery_icon() {
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        assert_eq!(resource.icon(), "battery");
    }

    // =========================================================================
    // Serde serialization tests (tagged format)
    // =========================================================================

    #[test]
    fn test_battery_serialization_format() {
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        let json = serde_json::to_string(&resource).expect("should serialize");

        // Verify tagged format
        assert!(
            json.contains(r#""type":"battery""#),
            "Should have type tag 'battery', got: {json}"
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
    fn test_battery_serde_roundtrip() {
        let original = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: ZzzResource = serde_json::from_str(&json).expect("should deserialize");

        let ZzzResource::Battery(resource) = deserialized;
        assert_eq!(resource.current, 200);
        assert_eq!(resource.max, 240);
        assert_eq!(resource.regen_rate_seconds, 360);
    }

    #[test]
    fn test_battery_serializes_full_at_as_utc_z() {
        // Locks the JSON datetime contract consumed by the frontend: the nested
        // `data.fullAt` is an RFC3339 string in UTC with a trailing `Z`.
        let ts = Timestamp::from_second(1_704_067_200).expect("valid timestamp");
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, ts, 360));
        let value = serde_json::to_value(&resource).expect("should serialize");
        assert_eq!(value["type"], "battery");
        assert_eq!(value["data"]["fullAt"], "2024-01-01T00:00:00Z");
    }

    // =========================================================================
    // Debug trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_debug() {
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        let debug = format!("{resource:?}");
        assert!(debug.contains("Battery"));
    }

    // =========================================================================
    // Clone trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_clone() {
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        let cloned = resource.clone();

        let ZzzResource::Battery(r) = cloned;
        assert_eq!(r.current, 200);
        assert_eq!(r.max, 240);
    }
}
