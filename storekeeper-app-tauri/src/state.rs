//! Application state management.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use storekeeper_core::{AppConfig, GameId, SecretsConfig, ensure_configs_exist};
use tokio::sync::RwLock;

use crate::clients::create_registry;
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

/// Inner state data protected by RwLock.
#[derive(Default)]
pub struct StateData {
    /// Cached resources from all games.
    pub resources: AllResources,

    /// Whether a refresh is currently in progress.
    pub refreshing: bool,

    /// Registry-based game clients.
    pub registry: GameClientRegistry,

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

        Self {
            inner: Arc::new(RwLock::new(StateData {
                resources: AllResources::default(),
                refreshing: false,
                registry,
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
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
