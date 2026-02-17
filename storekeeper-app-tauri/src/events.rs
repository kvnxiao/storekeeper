//! Centralized event definitions for Tauri backend-to-frontend communication.

use serde::Serialize;
use storekeeper_core::GameId;

/// All events emitted from the Tauri backend to the frontend.
#[derive(Debug, Clone, Copy)]
pub enum AppEvent {
    /// All resources have been fetched and updated.
    ResourcesUpdated,
    /// A manual refresh has started.
    RefreshStarted,
    /// A single game's resources have been fetched.
    GameResourceUpdated,
    /// Daily rewards have been claimed.
    DailyRewardClaimed,
}

impl AppEvent {
    /// Returns the event name string used for Tauri event emission.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResourcesUpdated => "resources-updated",
            Self::RefreshStarted => "refresh-started",
            Self::GameResourceUpdated => "game-resource-updated",
            Self::DailyRewardClaimed => "daily-reward-claimed",
        }
    }
}

/// Payload for per-game resource update events.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameResourcePayload<'a> {
    /// The game that was updated.
    pub game_id: GameId,
    /// The resource data for this game.
    pub data: &'a serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // AppEvent::as_str — verify each variant
    // =========================================================================

    #[test]
    fn event_resources_updated() {
        assert_eq!(AppEvent::ResourcesUpdated.as_str(), "resources-updated");
    }

    #[test]
    fn event_refresh_started() {
        assert_eq!(AppEvent::RefreshStarted.as_str(), "refresh-started");
    }

    #[test]
    fn event_game_resource_updated() {
        assert_eq!(
            AppEvent::GameResourceUpdated.as_str(),
            "game-resource-updated"
        );
    }

    #[test]
    fn event_daily_reward_claimed() {
        assert_eq!(
            AppEvent::DailyRewardClaimed.as_str(),
            "daily-reward-claimed"
        );
    }

    // =========================================================================
    // AppEvent — all events use lowercase kebab-case
    // =========================================================================

    #[test]
    fn all_events_are_kebab_case() {
        let events = [
            AppEvent::ResourcesUpdated,
            AppEvent::RefreshStarted,
            AppEvent::GameResourceUpdated,
            AppEvent::DailyRewardClaimed,
        ];
        for event in events {
            let s = event.as_str();
            assert!(
                !s.contains('_') && !s.contains(' ') && s == s.to_lowercase(),
                "Event {s:?} should be lowercase kebab-case"
            );
        }
    }

    // =========================================================================
    // GameResourcePayload serde — camelCase field names
    // =========================================================================

    #[test]
    fn payload_serializes_camel_case() {
        let data = serde_json::json!({"stamina": 160});
        let payload = GameResourcePayload {
            game_id: GameId::GenshinImpact,
            data: &data,
        };
        let json = serde_json::to_value(&payload).expect("should serialize");
        assert!(
            json.get("gameId").is_some(),
            "field should be camelCase `gameId`"
        );
        assert!(
            json.get("game_id").is_none(),
            "field should NOT be snake_case `game_id`"
        );
    }
}
