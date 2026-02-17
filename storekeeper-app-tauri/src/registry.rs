//! Game client registry for dynamic client management.

use std::collections::HashMap;

use storekeeper_core::{DynGameClient, GameId};
use tauri::{AppHandle, Emitter};

use crate::events::{AppEvent, GameResourcePayload};
use crate::provider_batch;

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

    /// Fetches resources from all registered clients with rate limit awareness.
    ///
    /// Games are grouped by API provider:
    /// - Games using the same provider are fetched sequentially to avoid rate limits
    /// - Different providers are fetched in parallel for efficiency
    ///
    /// Emits a per-game event after each successful fetch.
    /// Returns a map from game ID to the JSON-serialized resources.
    /// Clients that fail to fetch are logged and skipped.
    pub async fn fetch_all(&self, app_handle: &AppHandle) -> HashMap<GameId, serde_json::Value> {
        provider_batch::batch_by_provider(&self.clients, |game_id, client| {
            let app_handle = app_handle.clone();
            Box::pin(async move {
                let result = client.fetch_resources_json().await;

                if let Ok(ref resources) = result {
                    let payload = GameResourcePayload {
                        game_id,
                        data: resources.clone(),
                    };
                    let _ = app_handle.emit(AppEvent::GameResourceUpdated.as_str(), &payload);
                }

                (game_id, result)
            })
        })
        .await
    }
}

impl Default for GameClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;

    use super::*;

    type BoxError = Box<dyn std::error::Error + Send + Sync>;
    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    struct MockGameClient {
        id: GameId,
    }

    impl DynGameClient for MockGameClient {
        fn game_id(&self) -> GameId {
            self.id
        }

        fn game_name(&self) -> &'static str {
            "Mock Game"
        }

        fn fetch_resources_json(&self) -> BoxFuture<'_, Result<serde_json::Value, BoxError>> {
            Box::pin(async { Ok(serde_json::json!({"mock": true})) })
        }

        fn is_authenticated_dyn(&self) -> BoxFuture<'_, Result<bool, BoxError>> {
            Box::pin(async { Ok(true) })
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
