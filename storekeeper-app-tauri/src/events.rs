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
pub struct GameResourcePayload {
    /// The game that was updated.
    pub game_id: GameId,
    /// The resource data for this game.
    pub data: serde_json::Value,
}
