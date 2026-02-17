//! Per-game resource type identifiers.
//!
//! These enums represent resource types as configuration keys (for tracked resources
//! and notification settings). They serialize to snake_case strings matching
//! the serde format of the corresponding data enums in each game crate.

use serde::{Deserialize, Serialize};
use strum::AsRefStr;

/// Genshin Impact resource type identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum GenshinResourceType {
    /// Original Resin.
    Resin,
    /// Parametric Transformer cooldown.
    ParametricTransformer,
    /// Serenitea Pot Realm Currency.
    RealmCurrency,
    /// Expedition tracking.
    Expeditions,
}

impl GenshinResourceType {
    /// Returns a static slice of all variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Resin,
            Self::ParametricTransformer,
            Self::RealmCurrency,
            Self::Expeditions,
        ]
    }
}

/// Honkai: Star Rail resource type identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum HsrResourceType {
    /// Trailblaze Power.
    TrailblazePower,
}

impl HsrResourceType {
    /// Returns a static slice of all variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::TrailblazePower]
    }
}

/// Zenless Zone Zero resource type identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ZzzResourceType {
    /// Battery charge.
    Battery,
}

impl ZzzResourceType {
    /// Returns a static slice of all variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Battery]
    }
}

/// Wuthering Waves resource type identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum WuwaResourceType {
    /// Waveplates.
    Waveplates,
}

impl WuwaResourceType {
    /// Returns a static slice of all variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Waveplates]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genshin_resource_type_serde_roundtrip() {
        let rt = GenshinResourceType::Resin;
        let json = serde_json::to_string(&rt).expect("serialize");
        assert_eq!(json, r#""resin""#);
        let deserialized: GenshinResourceType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, rt);
    }

    #[test]
    fn genshin_resource_type_as_ref() {
        assert_eq!(GenshinResourceType::Resin.as_ref(), "resin");
        assert_eq!(
            GenshinResourceType::ParametricTransformer.as_ref(),
            "parametric_transformer"
        );
        assert_eq!(
            GenshinResourceType::RealmCurrency.as_ref(),
            "realm_currency"
        );
        assert_eq!(GenshinResourceType::Expeditions.as_ref(), "expeditions");
    }

    #[test]
    fn genshin_resource_type_all() {
        assert_eq!(GenshinResourceType::all().len(), 4);
    }

    #[test]
    fn hsr_resource_type_serde() {
        let rt = HsrResourceType::TrailblazePower;
        let json = serde_json::to_string(&rt).expect("serialize");
        assert_eq!(json, r#""trailblaze_power""#);
    }

    #[test]
    fn zzz_resource_type_serde() {
        let rt = ZzzResourceType::Battery;
        let json = serde_json::to_string(&rt).expect("serialize");
        assert_eq!(json, r#""battery""#);
    }

    #[test]
    fn wuwa_resource_type_serde() {
        let rt = WuwaResourceType::Waveplates;
        let json = serde_json::to_string(&rt).expect("serialize");
        assert_eq!(json, r#""waveplates""#);
    }

    #[derive(Deserialize)]
    struct TomlKeyWrapper {
        key: GenshinResourceType,
    }

    #[test]
    fn genshin_resource_type_toml_deserialization() {
        // Simulates TOML config key deserialization
        let toml_str = r#"key = "resin""#;
        let w: TomlKeyWrapper = toml::from_str(toml_str).expect("should parse");
        assert_eq!(w.key, GenshinResourceType::Resin);
    }

    #[derive(Deserialize)]
    struct NotifValue {
        value: i32,
    }

    #[derive(Deserialize)]
    struct TomlMapWrapper {
        data: std::collections::HashMap<GenshinResourceType, NotifValue>,
    }

    #[test]
    fn genshin_resource_type_toml_hashmap_key() {
        let toml_str = r"
            [data.resin]
            value = 1
            [data.expeditions]
            value = 2
        ";
        let w: TomlMapWrapper = toml::from_str(toml_str).expect("should parse");
        assert_eq!(w.data.len(), 2);
        assert_eq!(w.data[&GenshinResourceType::Resin].value, 1);
        assert_eq!(w.data[&GenshinResourceType::Expeditions].value, 2);
    }
}
