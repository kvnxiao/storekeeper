//! Zenless Zone Zero resource types.

use storekeeper_core::StaminaResource;
use storekeeper_core::game_resource_enum;

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

    #[test]
    fn battery_display_name() {
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        assert_eq!(resource.display_name(), "Battery");
    }

    #[test]
    fn battery_icon() {
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        assert_eq!(resource.icon(), "battery");
    }

    #[test]
    fn battery_serialization_format() {
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        let json = serde_json::to_string(&resource).expect("should serialize");

        assert!(
            json.contains(r#""type":"battery""#),
            "Should have type tag 'battery', got: {json}"
        );
        assert!(
            json.contains(r#""data":"#),
            "Should have data field, got: {json}"
        );
    }

    #[test]
    fn battery_serde_roundtrip() {
        let original = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: ZzzResource = serde_json::from_str(&json).expect("should deserialize");

        let ZzzResource::Battery(resource) = deserialized;
        assert_eq!(resource.current, 200);
        assert_eq!(resource.max, 240);
        assert_eq!(resource.regen_rate_seconds, 360);
    }

    #[test]
    fn battery_serializes_full_at_as_utc_z() {
        let ts = Timestamp::from_second(1_704_067_200).expect("valid timestamp");
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, ts, 360));
        let value = serde_json::to_value(&resource).expect("should serialize");
        assert_eq!(
            value.get("type").and_then(serde_json::Value::as_str),
            Some("battery")
        );
        assert_eq!(
            value
                .get("data")
                .and_then(|data| data.get("fullAt"))
                .and_then(serde_json::Value::as_str),
            Some("2024-01-01T00:00:00Z")
        );
    }

    #[test]
    fn resource_is_debug() {
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        let debug = format!("{resource:?}");
        assert!(debug.contains("Battery"));
    }

    #[test]
    fn resource_is_clone() {
        let resource = ZzzResource::Battery(StaminaResource::new(200, 240, Timestamp::now(), 360));
        let cloned = resource.clone();

        let ZzzResource::Battery(r) = cloned;
        assert_eq!(r.current, 200);
        assert_eq!(r.max, 240);
    }
}
