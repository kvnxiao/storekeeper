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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
            Self::GenshinImpact => "GENSHIN_IMPACT",
            Self::HonkaiStarRail => "HONKAI_STAR_RAIL",
            Self::ZenlessZoneZero => "ZENLESS_ZONE_ZERO",
            Self::WutheringWaves => "WUTHERING_WAVES",
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

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // ApiProvider tests
    // =========================================================================

    #[test]
    fn test_api_provider_for_hoyolab_games() {
        assert_eq!(
            GameId::GenshinImpact.api_provider(),
            ApiProvider::HoYoLab,
            "Genshin Impact should use HoYoLab API"
        );
        assert_eq!(
            GameId::HonkaiStarRail.api_provider(),
            ApiProvider::HoYoLab,
            "Honkai: Star Rail should use HoYoLab API"
        );
        assert_eq!(
            GameId::ZenlessZoneZero.api_provider(),
            ApiProvider::HoYoLab,
            "Zenless Zone Zero should use HoYoLab API"
        );
    }

    #[test]
    fn test_api_provider_for_kuro_games() {
        assert_eq!(
            GameId::WutheringWaves.api_provider(),
            ApiProvider::Kuro,
            "Wuthering Waves should use Kuro API"
        );
    }

    // =========================================================================
    // GameId::as_str tests
    // =========================================================================

    #[test]
    fn test_as_str_matches_serde_format() {
        assert_eq!(GameId::GenshinImpact.as_str(), "GENSHIN_IMPACT");
        assert_eq!(GameId::HonkaiStarRail.as_str(), "HONKAI_STAR_RAIL");
        assert_eq!(GameId::ZenlessZoneZero.as_str(), "ZENLESS_ZONE_ZERO");
        assert_eq!(GameId::WutheringWaves.as_str(), "WUTHERING_WAVES");
    }

    // =========================================================================
    // GameId::display_name tests
    // =========================================================================

    #[test]
    fn test_display_name_human_readable() {
        assert_eq!(GameId::GenshinImpact.display_name(), "Genshin Impact");
        assert_eq!(GameId::HonkaiStarRail.display_name(), "Honkai: Star Rail");
        assert_eq!(GameId::ZenlessZoneZero.display_name(), "Zenless Zone Zero");
        assert_eq!(GameId::WutheringWaves.display_name(), "Wuthering Waves");
    }

    // =========================================================================
    // GameId::all tests
    // =========================================================================

    #[test]
    fn test_all_returns_four_games() {
        let all = GameId::all();
        assert_eq!(all.len(), 4, "Should return exactly 4 games");
    }

    #[test]
    fn test_all_contains_all_variants() {
        let all = GameId::all();
        assert!(
            all.contains(&GameId::GenshinImpact),
            "Should contain Genshin Impact"
        );
        assert!(
            all.contains(&GameId::HonkaiStarRail),
            "Should contain Honkai: Star Rail"
        );
        assert!(
            all.contains(&GameId::ZenlessZoneZero),
            "Should contain Zenless Zone Zero"
        );
        assert!(
            all.contains(&GameId::WutheringWaves),
            "Should contain Wuthering Waves"
        );
    }

    #[test]
    fn test_all_is_in_expected_order() {
        let all = GameId::all();
        assert_eq!(all[0], GameId::GenshinImpact);
        assert_eq!(all[1], GameId::HonkaiStarRail);
        assert_eq!(all[2], GameId::ZenlessZoneZero);
        assert_eq!(all[3], GameId::WutheringWaves);
    }

    // =========================================================================
    // Display trait tests
    // =========================================================================

    #[test]
    fn test_display_uses_display_name() {
        assert_eq!(format!("{}", GameId::GenshinImpact), "Genshin Impact");
        assert_eq!(format!("{}", GameId::HonkaiStarRail), "Honkai: Star Rail");
        assert_eq!(format!("{}", GameId::ZenlessZoneZero), "Zenless Zone Zero");
        assert_eq!(format!("{}", GameId::WutheringWaves), "Wuthering Waves");
    }

    // =========================================================================
    // Serde roundtrip tests
    // =========================================================================

    #[test]
    fn test_serde_serialization() {
        let json =
            serde_json::to_string(&GameId::GenshinImpact).expect("should serialize Genshin Impact");
        assert_eq!(json, "\"GENSHIN_IMPACT\"");

        let json = serde_json::to_string(&GameId::HonkaiStarRail)
            .expect("should serialize Honkai: Star Rail");
        assert_eq!(json, "\"HONKAI_STAR_RAIL\"");

        let json = serde_json::to_string(&GameId::ZenlessZoneZero)
            .expect("should serialize Zenless Zone Zero");
        assert_eq!(json, "\"ZENLESS_ZONE_ZERO\"");

        let json = serde_json::to_string(&GameId::WutheringWaves)
            .expect("should serialize Wuthering Waves");
        assert_eq!(json, "\"WUTHERING_WAVES\"");
    }

    #[test]
    fn test_serde_deserialization() {
        let game: GameId =
            serde_json::from_str("\"GENSHIN_IMPACT\"").expect("should deserialize GENSHIN_IMPACT");
        assert_eq!(game, GameId::GenshinImpact);

        let game: GameId = serde_json::from_str("\"HONKAI_STAR_RAIL\"")
            .expect("should deserialize HONKAI_STAR_RAIL");
        assert_eq!(game, GameId::HonkaiStarRail);

        let game: GameId = serde_json::from_str("\"ZENLESS_ZONE_ZERO\"")
            .expect("should deserialize ZENLESS_ZONE_ZERO");
        assert_eq!(game, GameId::ZenlessZoneZero);

        let game: GameId = serde_json::from_str("\"WUTHERING_WAVES\"")
            .expect("should deserialize WUTHERING_WAVES");
        assert_eq!(game, GameId::WutheringWaves);
    }

    #[test]
    fn test_serde_roundtrip() {
        for game in GameId::all() {
            let json = serde_json::to_string(game).expect("should serialize");
            let deserialized: GameId = serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(*game, deserialized, "Roundtrip failed for {game:?}");
        }
    }

    #[test]
    fn test_serde_invalid_value() {
        let result: Result<GameId, _> = serde_json::from_str("\"invalid_game\"");
        assert!(
            result.is_err(),
            "Invalid game ID should fail to deserialize"
        );
    }

    // =========================================================================
    // Trait implementation tests
    // =========================================================================

    #[test]
    fn test_game_id_is_copy() {
        let game = GameId::GenshinImpact;
        let game2 = game; // Copy
        let game3 = game; // Copy again
        assert_eq!(game, game2);
        assert_eq!(game, game3);
    }

    #[test]
    fn test_game_id_eq() {
        assert_eq!(GameId::GenshinImpact, GameId::GenshinImpact);
        assert_ne!(GameId::GenshinImpact, GameId::HonkaiStarRail);
    }

    #[test]
    fn test_game_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(GameId::GenshinImpact);
        set.insert(GameId::HonkaiStarRail);
        set.insert(GameId::GenshinImpact); // Duplicate

        assert_eq!(set.len(), 2, "HashSet should have 2 unique games");
        assert!(set.contains(&GameId::GenshinImpact));
        assert!(set.contains(&GameId::HonkaiStarRail));
    }

    #[test]
    fn test_api_provider_eq() {
        assert_eq!(ApiProvider::HoYoLab, ApiProvider::HoYoLab);
        assert_eq!(ApiProvider::Kuro, ApiProvider::Kuro);
        assert_ne!(ApiProvider::HoYoLab, ApiProvider::Kuro);
    }

    #[test]
    fn test_api_provider_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ApiProvider::HoYoLab);
        set.insert(ApiProvider::Kuro);
        set.insert(ApiProvider::HoYoLab); // Duplicate

        assert_eq!(set.len(), 2, "HashSet should have 2 unique providers");
    }
}
