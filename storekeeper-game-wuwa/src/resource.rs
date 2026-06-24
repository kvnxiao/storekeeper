//! Wuthering Waves resource types.

use storekeeper_core::StaminaResource;
use storekeeper_core::game_resource_enum;

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
    use jiff::Timestamp;
    use storekeeper_core::DisplayableResource;

    // =========================================================================
    // DisplayableResource trait tests
    // =========================================================================

    #[test]
    fn test_waveplates_display_name() {
        let resource =
            WuwaResource::Waveplates(StaminaResource::new(120, 240, Timestamp::now(), 360));
        assert_eq!(resource.display_name(), "Waveplates");
    }

    #[test]
    fn test_waveplates_icon() {
        let resource =
            WuwaResource::Waveplates(StaminaResource::new(120, 240, Timestamp::now(), 360));
        assert_eq!(resource.icon(), "waveplate");
    }

    // =========================================================================
    // Serde serialization tests (tagged format)
    // =========================================================================

    #[test]
    fn test_waveplates_serialization_format() {
        let resource =
            WuwaResource::Waveplates(StaminaResource::new(120, 240, Timestamp::now(), 360));
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
        let original =
            WuwaResource::Waveplates(StaminaResource::new(120, 240, Timestamp::now(), 360));
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: WuwaResource = serde_json::from_str(&json).expect("should deserialize");

        let WuwaResource::Waveplates(resource) = deserialized;
        assert_eq!(resource.current, 120);
        assert_eq!(resource.max, 240);
        assert_eq!(resource.regen_rate_seconds, 360);
    }

    #[test]
    fn test_waveplates_serializes_full_at_as_utc_z() {
        // Locks the JSON datetime contract consumed by the frontend: the nested
        // `data.fullAt` is an RFC3339 string in UTC with a trailing `Z`.
        let ts = Timestamp::from_second(1_704_067_200).expect("valid timestamp");
        let resource = WuwaResource::Waveplates(StaminaResource::new(120, 240, ts, 360));
        let value = serde_json::to_value(&resource).expect("should serialize");
        assert_eq!(
            value.get("type").and_then(serde_json::Value::as_str),
            Some("waveplates")
        );
        assert_eq!(
            value
                .get("data")
                .and_then(|data| data.get("fullAt"))
                .and_then(serde_json::Value::as_str),
            Some("2024-01-01T00:00:00Z")
        );
    }

    // =========================================================================
    // Debug trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_debug() {
        let resource =
            WuwaResource::Waveplates(StaminaResource::new(120, 240, Timestamp::now(), 360));
        let debug = format!("{resource:?}");
        assert!(debug.contains("Waveplates"));
    }

    // =========================================================================
    // Clone trait tests
    // =========================================================================

    #[test]
    fn test_resource_is_clone() {
        let resource =
            WuwaResource::Waveplates(StaminaResource::new(120, 240, Timestamp::now(), 360));
        let cloned = resource.clone();

        let WuwaResource::Waveplates(r) = cloned;
        assert_eq!(r.current, 120);
        assert_eq!(r.max, 240);
    }
}
