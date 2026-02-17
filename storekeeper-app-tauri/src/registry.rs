//! Game client registry for dynamic client management.

use std::collections::{HashMap, HashSet};

use futures::future::join_all;
use storekeeper_core::{ApiProvider, DynGameClient, GameId};
use tauri::{AppHandle, Emitter};

use crate::events::{AppEvent, GameResourcePayload};

/// Registry that holds type-erased game clients.
///
/// This allows storing different game client types in a single collection,
/// enabling dynamic iteration and fetching without explicit per-game fields.
pub struct GameClientRegistry {
    clients: HashMap<GameId, Box<dyn DynGameClient>>,
}

impl GameClientRegistry {
    /// Creates a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Registers a game client in the registry.
    ///
    /// If a client for the same game already exists, it will be replaced.
    pub fn register(&mut self, client: Box<dyn DynGameClient>) {
        let id = client.game_id();
        tracing::debug!(game_id = ?id, "Registering game client");
        self.clients.insert(id, client);
    }

    /// Returns the number of registered clients.
    #[must_use]
    pub fn len(&self) -> usize {
        self.clients.len()
    }

    /// Returns true if any clients are registered.
    #[must_use]
    pub fn has_any(&self) -> bool {
        !self.clients.is_empty()
    }

    /// Fetches resources from clients for a specific API provider sequentially.
    ///
    /// This avoids rate limiting by fetching one game at a time for the same provider.
    /// Emits a per-game event after each successful fetch.
    async fn fetch_provider(
        &self,
        provider: ApiProvider,
        app_handle: &AppHandle,
    ) -> Vec<(
        GameId,
        Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>,
    )> {
        let mut results = Vec::new();

        for (game_id, client) in &self.clients {
            if game_id.api_provider() == provider {
                let result = client.fetch_resources_json().await;

                if let Ok(ref resources) = result {
                    let payload = GameResourcePayload {
                        game_id: *game_id,
                        data: resources.clone(),
                    };
                    let _ = app_handle.emit(AppEvent::GameResourceUpdated.as_str(), &payload);
                }

                results.push((*game_id, result));
            }
        }

        results
    }

    /// Fetches resources from all registered clients with rate limit awareness.
    ///
    /// Games are grouped by API provider:
    /// - Games using the same provider are fetched sequentially to avoid rate limits
    /// - Different providers are fetched in parallel for efficiency
    ///
    /// Returns a map from game ID to the JSON-serialized resources.
    /// Clients that fail to fetch are logged and skipped.
    pub async fn fetch_all(&self, app_handle: &AppHandle) -> HashMap<GameId, serde_json::Value> {
        if self.clients.is_empty() {
            return HashMap::new();
        }

        // Determine which providers have clients registered
        let providers: HashSet<ApiProvider> = self
            .clients
            .keys()
            .map(storekeeper_core::GameId::api_provider)
            .collect();

        // Fetch each provider's games in parallel, but games within a provider sequentially
        let provider_futures: Vec<_> = providers
            .iter()
            .map(|provider| self.fetch_provider(*provider, app_handle))
            .collect();

        let all_results = join_all(provider_futures).await;

        // Collect all results into the map
        let mut map = HashMap::new();
        for provider_results in all_results {
            for (game_id, result) in provider_results {
                match result {
                    Ok(resources) => {
                        tracing::debug!(game_id = ?game_id, "Successfully fetched resources");
                        map.insert(game_id, resources);
                    }
                    Err(e) => {
                        tracing::warn!(game_id = ?game_id, error = %e, "Failed to fetch resources");
                    }
                }
            }
        }

        map
    }
}

impl Default for GameClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockGameClient {
        id: GameId,
    }

    #[async_trait]
    impl DynGameClient for MockGameClient {
        fn game_id(&self) -> GameId {
            self.id
        }

        fn game_name(&self) -> &'static str {
            "Mock Game"
        }

        async fn fetch_resources_json(
            &self,
        ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            Ok(serde_json::json!({"mock": true}))
        }

        async fn is_authenticated_dyn(
            &self,
        ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            Ok(true)
        }
    }

    // =========================================================================
    // Construction
    // =========================================================================

    #[test]
    fn new_registry_is_empty() {
        let r = GameClientRegistry::new();
        assert_eq!(r.len(), 0);
        assert!(!r.has_any());
    }

    #[test]
    fn default_registry_is_empty() {
        let r = GameClientRegistry::default();
        assert_eq!(r.len(), 0);
        assert!(!r.has_any());
    }

    // =========================================================================
    // Registration
    // =========================================================================

    #[test]
    fn register_single_client() {
        let mut r = GameClientRegistry::new();
        r.register(Box::new(MockGameClient {
            id: GameId::GenshinImpact,
        }));
        assert_eq!(r.len(), 1);
        assert!(r.has_any());
    }

    #[test]
    fn register_multiple_clients() {
        let mut r = GameClientRegistry::new();
        r.register(Box::new(MockGameClient {
            id: GameId::GenshinImpact,
        }));
        r.register(Box::new(MockGameClient {
            id: GameId::HonkaiStarRail,
        }));
        r.register(Box::new(MockGameClient {
            id: GameId::WutheringWaves,
        }));
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn duplicate_game_id_replaces() {
        let mut r = GameClientRegistry::new();
        r.register(Box::new(MockGameClient {
            id: GameId::GenshinImpact,
        }));
        r.register(Box::new(MockGameClient {
            id: GameId::GenshinImpact,
        }));
        assert_eq!(r.len(), 1, "duplicate should replace, not add");
    }

    #[test]
    fn register_all_four_games() {
        let mut r = GameClientRegistry::new();
        for &id in GameId::all() {
            r.register(Box::new(MockGameClient { id }));
        }
        assert_eq!(r.len(), 4);
    }
}
