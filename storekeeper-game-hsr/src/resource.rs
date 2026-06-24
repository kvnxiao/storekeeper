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
    use jiff::Timestamp;
    use storekeeper_core::DisplayableResource;

    // =========================================================================
    // DisplayableResource trait tests
    // =========================================================================

    #[test]
    fn test_trailblaze_power_display_name() {
        let resource =
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Timestamp::now(), 360));
        assert_eq!(resource.display_name(), "Trailblaze Power");
    }

    #[test]
    fn test_trailblaze_power_icon() {
        let resource =
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Timestamp::now(), 360));
        assert_eq!(resource.icon(), "power");
    }

    // =========================================================================
    // Serde serialization tests (tagged format)
    // =========================================================================

    #[test]
    fn test_trailblaze_power_serialization_format() {
        let resource =
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Timestamp::now(), 360));
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
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Timestamp::now(), 360));
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: HsrResource = serde_json::from_str(&json).expect("should deserialize");

        let HsrResource::TrailblazePower(resource) = deserialized;
        assert_eq!(resource.current, 180);
        assert_eq!(resource.max, 240);
        assert_eq!(resource.regen_rate_seconds, 360);
    }

    #[test]
    fn test_trailblaze_power_serializes_full_at_as_utc_z() {
        // Locks the JSON datetime contract consumed by the frontend: the nested
        // `data.fullAt` is an RFC3339 string in UTC with a trailing `Z`.
        let ts = Timestamp::from_second(1_704_067_200).expect("valid timestamp");
        let resource = HsrResource::TrailblazePower(StaminaResource::new(180, 240, ts, 360));
        let value = serde_json::to_value(&resource).expect("should serialize");
        assert_eq!(value["type"], "trailblaze_power");
        assert_eq!(value["data"]["fullAt"], "2024-01-01T00:00:00Z");
    }

    // =========================================================================
    // Debug trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_debug() {
        let resource =
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Timestamp::now(), 360));
        let debug = format!("{resource:?}");
        assert!(debug.contains("TrailblazePower"));
    }

    // =========================================================================
    // Clone trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_clone() {
        let resource =
            HsrResource::TrailblazePower(StaminaResource::new(180, 240, Timestamp::now(), 360));
        let cloned = resource.clone();

        let HsrResource::TrailblazePower(r) = cloned;
        assert_eq!(r.current, 180);
        assert_eq!(r.max, 240);
    }
}
