//! Game identifier enum for type-safe game identification.

use serde::{Deserialize, Serialize};

/// API provider for a game.
///
/// Games are grouped by their API provider to enable rate limit management.
/// Games using the same provider should be fetched sequentially to avoid rate limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiProvider {
    /// HoYoLab API (miHoYo/HoYoverse games)
    HoYoLab,
    /// Kuro Games API
    Kuro,
}

/// Unique identifier for each supported game.
///
/// This enum provides a type-safe way to identify games throughout the application,
/// replacing string-based identification with compile-time checked values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameId {
    /// Genshin Impact
    GenshinImpact,
    /// Honkai: Star Rail
    HonkaiStarRail,
    /// Zenless Zone Zero
    ZenlessZoneZero,
    /// Wuthering Waves
    WutheringWaves,
}

impl GameId {
    /// Returns the API provider for this game.
    ///
    /// This is used to group games by provider for rate limit management.
    #[must_use]
    pub const fn api_provider(&self) -> ApiProvider {
        match self {
            Self::GenshinImpact | Self::HonkaiStarRail | Self::ZenlessZoneZero => {
                ApiProvider::HoYoLab
            }
            Self::WutheringWaves => ApiProvider::Kuro,
        }
    }

    /// Returns the string identifier for this game.
    ///
    /// This matches the serialized form of the enum variant.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::GenshinImpact => "genshin_impact",
            Self::HonkaiStarRail => "honkai_star_rail",
            Self::ZenlessZoneZero => "zenless_zone_zero",
            Self::WutheringWaves => "wuthering_waves",
        }
    }

    /// Returns the human-readable display name for this game.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::GenshinImpact => "Genshin Impact",
            Self::HonkaiStarRail => "Honkai: Star Rail",
            Self::ZenlessZoneZero => "Zenless Zone Zero",
            Self::WutheringWaves => "Wuthering Waves",
        }
    }

    /// Returns all game IDs in a fixed order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::GenshinImpact,
            Self::HonkaiStarRail,
            Self::ZenlessZoneZero,
            Self::WutheringWaves,
        ]
    }
}

impl std::fmt::Display for GameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
