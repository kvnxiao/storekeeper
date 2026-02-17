//! Daily reward client registry for managing reward claiming across games.

use std::collections::HashMap;

use anyhow::Context;
use storekeeper_core::{DynDailyRewardClient, GameId};

use crate::provider_batch;

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
    #[allow(dead_code)]
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
    pub async fn get_status_for_game(&self, game_id: GameId) -> anyhow::Result<serde_json::Value> {
        let client = self
            .clients
            .get(&game_id)
            .ok_or_else(|| anyhow::anyhow!("Game {game_id:?} not registered for daily rewards"))?;

        client
            .get_reward_status_json()
            .await
            .map_err(|e| anyhow::anyhow!(e))
            .context("failed to fetch daily reward status")
    }

    /// Claims daily reward for a specific game.
    ///
    /// # Errors
    ///
    /// Returns an error if the game is not registered or the claim fails.
    pub async fn claim_for_game(&self, game_id: GameId) -> anyhow::Result<serde_json::Value> {
        let client = self
            .clients
            .get(&game_id)
            .ok_or_else(|| anyhow::anyhow!("Game {game_id:?} not registered for daily rewards"))?;

        client
            .claim_daily_reward_json()
            .await
            .map_err(|e| anyhow::anyhow!(e))
            .context("failed to claim daily reward")
    }

    /// Gets reward status from all registered clients with rate limit awareness.
    ///
    /// Returns a map from game ID to the JSON-serialized reward status.
    pub async fn get_all_status(&self) -> HashMap<GameId, serde_json::Value> {
        provider_batch::batch_by_provider(&self.clients, |game_id, client| {
            Box::pin(async move { (game_id, client.get_reward_status_json().await) })
        })
        .await
    }

    /// Claims rewards from all registered clients with rate limit awareness.
    ///
    /// Returns a map from game ID to the JSON-serialized claim results.
    pub async fn claim_all(&self) -> HashMap<GameId, serde_json::Value> {
        provider_batch::batch_by_provider(&self.clients, |game_id, client| {
            Box::pin(async move {
                let result = client.claim_daily_reward_json().await;
                // Small delay between claims to avoid rate limiting
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                (game_id, result)
            })
        })
        .await
    }
}

impl Default for DailyRewardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;

    use super::*;
    use storekeeper_core::ApiProvider;

    type BoxError = Box<dyn std::error::Error + Send + Sync>;
    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    // =========================================================================
    // Mock
    // =========================================================================

    struct MockDailyRewardClient {
        id: GameId,
        should_fail: bool,
    }

    impl MockDailyRewardClient {
        fn success(id: GameId) -> Self {
            Self {
                id,
                should_fail: false,
            }
        }

        fn failing(id: GameId) -> Self {
            Self {
                id,
                should_fail: true,
            }
        }
    }

    impl DynDailyRewardClient for MockDailyRewardClient {
        fn game_id(&self) -> GameId {
            self.id
        }

        fn get_reward_status_json(&self) -> BoxFuture<'_, Result<serde_json::Value, BoxError>> {
            let should_fail = self.should_fail;
            Box::pin(async move {
                if should_fail {
                    Err("mock status error".into())
                } else {
                    Ok(serde_json::json!({"info": {"is_signed": true, "total_sign_day": 10}}))
                }
            })
        }

        fn claim_daily_reward_json(&self) -> BoxFuture<'_, Result<serde_json::Value, BoxError>> {
            let should_fail = self.should_fail;
            Box::pin(async move {
                if should_fail {
                    Err("mock claim error".into())
                } else {
                    Ok(serde_json::json!({"success": true}))
                }
            })
        }
    }

    // =========================================================================
    // Sync — construction & registration
    // =========================================================================

    #[test]
    fn new_registry_is_empty() {
        let r = DailyRewardRegistry::new();
        assert_eq!(r.len(), 0);
        assert!(r.is_empty());
    }

    #[test]
    fn default_registry_is_empty() {
        let r = DailyRewardRegistry::default();
        assert_eq!(r.len(), 0);
        assert!(r.is_empty());
    }

    #[test]
    fn register_and_has_game() {
        let mut r = DailyRewardRegistry::new();
        r.register(Box::new(MockDailyRewardClient::success(
            GameId::GenshinImpact,
        )));
        assert_eq!(r.len(), 1);
        assert!(!r.is_empty());
        assert!(r.has_game(GameId::GenshinImpact));
        assert!(!r.has_game(GameId::HonkaiStarRail));
    }

    #[test]
    fn duplicate_replaces() {
        let mut r = DailyRewardRegistry::new();
        r.register(Box::new(MockDailyRewardClient::success(
            GameId::GenshinImpact,
        )));
        r.register(Box::new(MockDailyRewardClient::failing(
            GameId::GenshinImpact,
        )));
        assert_eq!(r.len(), 1);
    }

    // =========================================================================
    // Async — get_status_for_game
    // =========================================================================

    #[tokio::test(start_paused = true)]
    async fn status_for_game_success() {
        let mut r = DailyRewardRegistry::new();
        r.register(Box::new(MockDailyRewardClient::success(
            GameId::GenshinImpact,
        )));
        let result = r.get_status_for_game(GameId::GenshinImpact).await;
        assert!(result.is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn status_for_game_not_registered() {
        let r = DailyRewardRegistry::new();
        let result = r.get_status_for_game(GameId::GenshinImpact).await;
        let err = result.expect_err("should fail for unregistered game");
        assert!(err.to_string().contains("not registered"));
    }

    #[tokio::test(start_paused = true)]
    async fn status_for_game_api_error() {
        let mut r = DailyRewardRegistry::new();
        r.register(Box::new(MockDailyRewardClient::failing(
            GameId::GenshinImpact,
        )));
        let result = r.get_status_for_game(GameId::GenshinImpact).await;
        let err = result.expect_err("should fail for mock API error");
        assert!(
            format!("{err:#}").contains("mock status error"),
            "full error chain should contain mock error message"
        );
    }

    // =========================================================================
    // Async — claim_for_game
    // =========================================================================

    #[tokio::test(start_paused = true)]
    async fn claim_for_game_success() {
        let mut r = DailyRewardRegistry::new();
        r.register(Box::new(MockDailyRewardClient::success(
            GameId::HonkaiStarRail,
        )));
        let result = r.claim_for_game(GameId::HonkaiStarRail).await;
        assert!(result.is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn claim_for_game_not_registered() {
        let r = DailyRewardRegistry::new();
        let result = r.claim_for_game(GameId::HonkaiStarRail).await;
        assert!(result.is_err());
    }

    // =========================================================================
    // Async — get_all_status
    // =========================================================================

    #[tokio::test(start_paused = true)]
    async fn get_all_status_empty() {
        let r = DailyRewardRegistry::new();
        let map = r.get_all_status().await;
        assert!(map.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn get_all_status_with_clients() {
        let mut r = DailyRewardRegistry::new();
        r.register(Box::new(MockDailyRewardClient::success(
            GameId::GenshinImpact,
        )));
        r.register(Box::new(MockDailyRewardClient::success(
            GameId::HonkaiStarRail,
        )));
        let map = r.get_all_status().await;
        assert_eq!(map.len(), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn get_all_status_partial_failure() {
        let mut r = DailyRewardRegistry::new();
        r.register(Box::new(MockDailyRewardClient::success(
            GameId::GenshinImpact,
        )));
        r.register(Box::new(MockDailyRewardClient::failing(
            GameId::HonkaiStarRail,
        )));
        let map = r.get_all_status().await;
        assert_eq!(map.len(), 1, "only successful status collected");
    }

    // =========================================================================
    // Async — claim_all
    // =========================================================================

    #[tokio::test(start_paused = true)]
    async fn claim_all_empty() {
        let r = DailyRewardRegistry::new();
        let map = r.claim_all().await;
        assert!(map.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn claim_all_with_clients() {
        let mut r = DailyRewardRegistry::new();
        r.register(Box::new(MockDailyRewardClient::success(
            GameId::GenshinImpact,
        )));
        r.register(Box::new(MockDailyRewardClient::success(
            GameId::ZenlessZoneZero,
        )));
        let map = r.claim_all().await;
        assert_eq!(map.len(), 2);
    }

    // =========================================================================
    // Provider grouping (verifying the batch_by_provider behavior)
    // =========================================================================

    #[test]
    fn verify_provider_grouping() {
        // Ensure HoYoLab games share a provider
        assert_eq!(GameId::GenshinImpact.api_provider(), ApiProvider::HoYoLab);
        assert_eq!(GameId::HonkaiStarRail.api_provider(), ApiProvider::HoYoLab);
        assert_eq!(GameId::ZenlessZoneZero.api_provider(), ApiProvider::HoYoLab);
        // WuWa is separate
        assert_eq!(GameId::WutheringWaves.api_provider(), ApiProvider::Kuro);
    }
}
