//! Daily reward client registry for managing reward claiming across games.

use std::collections::HashMap;

use futures::future::join_all;
use storekeeper_core::{ApiProvider, DynDailyRewardClient, GameId};

/// Type alias for the result of a daily reward operation.
type DailyRewardResult = (
    GameId,
    Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>,
);

/// Registry that holds type-erased daily reward clients.
///
/// Similar to `GameClientRegistry`, this allows storing different game clients
/// that implement daily reward functionality in a single collection.
pub struct DailyRewardRegistry {
    clients: HashMap<GameId, Box<dyn DynDailyRewardClient>>,
}

impl DailyRewardRegistry {
    /// Creates a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Registers a daily reward client in the registry.
    ///
    /// If a client for the same game already exists, it will be replaced.
    pub fn register(&mut self, client: Box<dyn DynDailyRewardClient>) {
        let id = client.game_id();
        tracing::debug!(game_id = ?id, "Registering daily reward client");
        self.clients.insert(id, client);
    }

    /// Returns the number of registered clients.
    #[must_use]
    pub fn len(&self) -> usize {
        self.clients.len()
    }

    /// Returns true if no clients are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    /// Returns true if a specific game is registered.
    #[must_use]
    pub fn has_game(&self, game_id: GameId) -> bool {
        self.clients.contains_key(&game_id)
    }

    /// Gets the reward status for a specific game.
    ///
    /// # Errors
    ///
    /// Returns an error if the game is not registered or the fetch fails.
    pub async fn get_status_for_game(&self, game_id: GameId) -> Result<serde_json::Value, String> {
        let client = self
            .clients
            .get(&game_id)
            .ok_or_else(|| format!("Game {game_id:?} not registered for daily rewards"))?;

        client
            .get_reward_status_json()
            .await
            .map_err(|e| e.to_string())
    }

    /// Claims daily reward for a specific game.
    ///
    /// # Errors
    ///
    /// Returns an error if the game is not registered or the claim fails.
    pub async fn claim_for_game(&self, game_id: GameId) -> Result<serde_json::Value, String> {
        let client = self
            .clients
            .get(&game_id)
            .ok_or_else(|| format!("Game {game_id:?} not registered for daily rewards"))?;

        client
            .claim_daily_reward_json()
            .await
            .map_err(|e| e.to_string())
    }

    /// Gets reward status from clients for a specific API provider sequentially.
    async fn get_status_provider(&self, provider: ApiProvider) -> Vec<DailyRewardResult> {
        let mut results = Vec::new();

        for (game_id, client) in &self.clients {
            if game_id.api_provider() == provider {
                let result = client.get_reward_status_json().await;
                results.push((*game_id, result));
            }
        }

        results
    }

    /// Claims rewards from clients for a specific API provider sequentially.
    async fn claim_provider(&self, provider: ApiProvider) -> Vec<DailyRewardResult> {
        let mut results = Vec::new();

        for (game_id, client) in &self.clients {
            if game_id.api_provider() == provider {
                let result = client.claim_daily_reward_json().await;
                results.push((*game_id, result));

                // Small delay between claims to avoid rate limiting
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }

        results
    }

    /// Gets reward status from all registered clients with rate limit awareness.
    ///
    /// Returns a map from game ID to the JSON-serialized reward status.
    pub async fn get_all_status(&self) -> HashMap<GameId, serde_json::Value> {
        if self.clients.is_empty() {
            return HashMap::new();
        }

        let providers = self.get_active_providers();

        // Fetch each provider's games in parallel
        let provider_futures: Vec<_> = providers
            .iter()
            .map(|provider| self.get_status_provider(*provider))
            .collect();

        let all_results = join_all(provider_futures).await;
        Self::collect_results(all_results)
    }

    /// Claims rewards from all registered clients with rate limit awareness.
    ///
    /// Returns a map from game ID to the JSON-serialized claim results.
    pub async fn claim_all(&self) -> HashMap<GameId, serde_json::Value> {
        if self.clients.is_empty() {
            return HashMap::new();
        }

        let providers = self.get_active_providers();

        // Claim each provider's games in parallel, but games within a provider sequentially
        let provider_futures: Vec<_> = providers
            .iter()
            .map(|provider| self.claim_provider(*provider))
            .collect();

        let all_results = join_all(provider_futures).await;
        Self::collect_results(all_results)
    }

    /// Gets the list of providers that have registered clients.
    fn get_active_providers(&self) -> Vec<ApiProvider> {
        let mut providers = Vec::new();
        if self
            .clients
            .keys()
            .any(|id| id.api_provider() == ApiProvider::HoYoLab)
        {
            providers.push(ApiProvider::HoYoLab);
        }
        if self
            .clients
            .keys()
            .any(|id| id.api_provider() == ApiProvider::Kuro)
        {
            providers.push(ApiProvider::Kuro);
        }
        providers
    }

    /// Collects results from provider fetches into a single map.
    fn collect_results(
        all_results: Vec<Vec<DailyRewardResult>>,
    ) -> HashMap<GameId, serde_json::Value> {
        let mut map = HashMap::new();
        for provider_results in all_results {
            for (game_id, result) in provider_results {
                match result {
                    Ok(data) => {
                        tracing::debug!(game_id = ?game_id, "Successfully processed daily reward operation");
                        map.insert(game_id, data);
                    }
                    Err(e) => {
                        tracing::warn!(game_id = ?game_id, error = %e, "Failed daily reward operation");
                    }
                }
            }
        }
        map
    }
}

impl Default for DailyRewardRegistry {
    fn default() -> Self {
        Self::new()
    }
}
