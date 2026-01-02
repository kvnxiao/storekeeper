//! Application state management.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use storekeeper_core::{AppConfig, ClaimTime, GameId, SecretsConfig, ensure_configs_exist};

use tokio::sync::RwLock;

use crate::clients::{create_daily_reward_registry, create_registry};
use crate::daily_reward_registry::DailyRewardRegistry;
use crate::registry::GameClientRegistry;

/// All resources from all games.
///
/// Resources are stored as a map from game ID to JSON value, allowing
/// for dynamic game support without explicit per-game fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllResources {
    /// Resources keyed by game ID.
    ///
    /// Each value is a JSON array of the game's resource types.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub games: HashMap<GameId, serde_json::Value>,

    /// Last update timestamp.
    pub last_updated: Option<DateTime<Utc>>,
}

/// All daily reward status from all games.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllDailyRewardStatus {
    /// Reward status keyed by game ID.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub games: HashMap<GameId, serde_json::Value>,

    /// Last check timestamp.
    pub last_checked: Option<DateTime<Utc>>,
}

/// Inner state data protected by RwLock.
#[derive(Default)]
pub struct StateData {
    /// Cached resources from all games.
    pub resources: AllResources,

    /// Whether a refresh is currently in progress.
    pub refreshing: bool,

    /// Registry-based game clients.
    pub registry: GameClientRegistry,

    /// Daily reward client registry.
    pub daily_reward_registry: DailyRewardRegistry,

    /// Cached daily reward status.
    pub daily_reward_status: AllDailyRewardStatus,

    /// Application configuration.
    pub config: AppConfig,
}

/// Application state wrapper.
#[derive(Clone)]
pub struct AppState {
    /// Inner state protected by async RwLock.
    pub inner: Arc<RwLock<StateData>>,
}

impl AppState {
    /// Creates a new application state with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(StateData::default())),
        }
    }

    /// Creates a new application state with initialized clients from config.
    ///
    /// Attempts to load configuration and secrets files. If they don't exist,
    /// creates default config files first, then loads them.
    #[must_use]
    pub fn with_config() -> Self {
        // Ensure config files exist, creating defaults if needed
        if let Err(e) = ensure_configs_exist() {
            tracing::warn!("Failed to ensure config files exist: {e}");
        }

        let config = AppConfig::load().unwrap_or_default();
        let secrets = SecretsConfig::load().unwrap_or_default();

        let registry = create_registry(&config, &secrets);
        let daily_reward_registry = create_daily_reward_registry(&config, &secrets);

        Self {
            inner: Arc::new(RwLock::new(StateData {
                resources: AllResources::default(),
                refreshing: false,
                registry,
                daily_reward_registry,
                daily_reward_status: AllDailyRewardStatus::default(),
                config,
            })),
        }
    }

    /// Gets a clone of the current resources.
    pub async fn get_resources(&self) -> AllResources {
        let state = self.inner.read().await;
        state.resources.clone()
    }

    /// Updates the resources.
    pub async fn set_resources(&self, resources: AllResources) {
        let mut state = self.inner.write().await;
        state.resources = resources;
    }

    /// Checks if a refresh is in progress.
    pub async fn is_refreshing(&self) -> bool {
        let state = self.inner.read().await;
        state.refreshing
    }

    /// Sets the refreshing flag.
    pub async fn set_refreshing(&self, refreshing: bool) {
        let mut state = self.inner.write().await;
        state.refreshing = refreshing;
    }

    /// Fetches resources from all configured game clients using the registry.
    pub async fn fetch_all_resources(&self) -> AllResources {
        let state = self.inner.read().await;
        let games = state.registry.fetch_all().await;
        AllResources {
            games,
            last_updated: Some(Utc::now()),
        }
    }

    /// Returns the poll interval from config.
    pub async fn poll_interval_secs(&self) -> u64 {
        let state = self.inner.read().await;
        state.config.general.poll_interval_secs
    }

    /// Returns whether any game clients are configured.
    pub async fn has_clients(&self) -> bool {
        let state = self.inner.read().await;
        state.registry.has_any()
    }

    // ========================================================================
    // Daily Reward Methods
    // ========================================================================

    /// Gets the cached daily reward status.
    pub async fn get_daily_reward_status(&self) -> AllDailyRewardStatus {
        let state = self.inner.read().await;
        state.daily_reward_status.clone()
    }

    /// Updates the cached daily reward status.
    pub async fn set_daily_reward_status(&self, status: AllDailyRewardStatus) {
        let mut state = self.inner.write().await;
        state.daily_reward_status = status;
    }

    /// Fetches daily reward status from all configured games.
    pub async fn fetch_all_daily_reward_status(&self) -> AllDailyRewardStatus {
        let state = self.inner.read().await;
        let games = state.daily_reward_registry.get_all_status().await;
        AllDailyRewardStatus {
            games,
            last_checked: Some(Utc::now()),
        }
    }

    /// Claims daily rewards from all configured games.
    pub async fn claim_all_daily_rewards(&self) -> HashMap<GameId, serde_json::Value> {
        let state = self.inner.read().await;
        state.daily_reward_registry.claim_all().await
    }

    /// Claims daily reward for a specific game.
    ///
    /// # Errors
    ///
    /// Returns an error if the game is not configured or the claim fails.
    pub async fn claim_daily_reward_for_game(
        &self,
        game_id: GameId,
    ) -> Result<serde_json::Value, String> {
        let state = self.inner.read().await;
        state.daily_reward_registry.claim_for_game(game_id).await
    }

    /// Gets the daily reward status for a specific game.
    ///
    /// # Errors
    ///
    /// Returns an error if the game is not configured or the fetch fails.
    pub async fn get_daily_reward_status_for_game(
        &self,
        game_id: GameId,
    ) -> Result<serde_json::Value, String> {
        let state = self.inner.read().await;
        state
            .daily_reward_registry
            .get_status_for_game(game_id)
            .await
    }

    /// Gets the list of games that have auto-claim enabled.
    ///
    /// Returns a list of `(GameId, Option<ClaimTime>)` pairs.
    pub async fn get_auto_claim_games(&self) -> Vec<(GameId, Option<ClaimTime>)> {
        let state = self.inner.read().await;
        let mut games = Vec::new();

        if let Some(ref cfg) = state.config.games.genshin_impact {
            if cfg.enabled && cfg.auto_claim_daily_rewards {
                games.push((GameId::GenshinImpact, cfg.auto_claim_time));
            }
        }
        if let Some(ref cfg) = state.config.games.honkai_star_rail {
            if cfg.enabled && cfg.auto_claim_daily_rewards {
                games.push((GameId::HonkaiStarRail, cfg.auto_claim_time));
            }
        }
        if let Some(ref cfg) = state.config.games.zenless_zone_zero {
            if cfg.enabled && cfg.auto_claim_daily_rewards {
                games.push((GameId::ZenlessZoneZero, cfg.auto_claim_time));
            }
        }

        games
    }

    /// Checks if auto-claim is enabled for a specific game.
    ///
    /// Returns true if auto-claim is enabled in config and the game is registered
    /// in the daily reward registry. Does not check if already claimed today -
    /// that is determined by fetching status from the API.
    pub async fn should_auto_claim_game(&self, game_id: GameId) -> bool {
        let state = self.inner.read().await;

        // Check if the game has auto-claim enabled
        let auto_claim_enabled = match game_id {
            GameId::GenshinImpact => state
                .config
                .games
                .genshin_impact
                .as_ref()
                .is_some_and(|c| c.enabled && c.auto_claim_daily_rewards),
            GameId::HonkaiStarRail => state
                .config
                .games
                .honkai_star_rail
                .as_ref()
                .is_some_and(|c| c.enabled && c.auto_claim_daily_rewards),
            GameId::ZenlessZoneZero => state
                .config
                .games
                .zenless_zone_zero
                .as_ref()
                .is_some_and(|c| c.enabled && c.auto_claim_daily_rewards),
            GameId::WutheringWaves => false, // Kuro games don't support daily rewards
        };

        if !auto_claim_enabled {
            return false;
        }

        // Check if the game is registered in the daily reward registry
        state.daily_reward_registry.has_game(game_id)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
