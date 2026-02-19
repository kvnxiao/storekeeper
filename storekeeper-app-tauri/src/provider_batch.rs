//! Batch-by-provider helper for executing operations across game clients.
//!
//! Groups clients by API provider and runs each provider's operations in parallel,
//! while operations within a single provider run sequentially (to respect rate limits).

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;

use futures::future::join_all;
use storekeeper_core::GameId;

/// Type alias for the result of a per-game operation.
type OperationResult = (
    GameId,
    Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>,
);

/// Executes an async operation on each client, batched by API provider.
///
/// Clients sharing an API provider are processed sequentially (to avoid rate limits).
/// Different providers are processed in parallel.
///
/// An optional `game_filter` limits which games are processed. When `None`, all
/// clients are processed.
///
/// The `operation` closure receives the game ID and client reference, and returns
/// a pinned future yielding the game ID paired with the result. The closure is
/// responsible for any side effects like event emission or inter-operation delays.
pub async fn batch_by_provider<C, F>(
    clients: &HashMap<GameId, Box<C>>,
    game_filter: Option<&HashSet<GameId>>,
    operation: F,
) -> HashMap<GameId, serde_json::Value>
where
    C: ?Sized + Sync,
    F: Fn(GameId, &C) -> Pin<Box<dyn Future<Output = OperationResult> + Send + '_>>,
{
    if clients.is_empty() {
        return HashMap::new();
    }

    let active_clients: Vec<_> = clients
        .iter()
        .filter(|(id, _)| game_filter.is_none_or(|f| f.contains(id)))
        .collect();

    if active_clients.is_empty() {
        return HashMap::new();
    }

    let providers: HashSet<_> = active_clients
        .iter()
        .map(|(id, _)| id.api_provider())
        .collect();

    let provider_futures: Vec<_> = providers
        .into_iter()
        .map(|provider| {
            let provider_clients: Vec<_> = active_clients
                .iter()
                .filter(|(id, _)| id.api_provider() == provider)
                .collect();

            async {
                let mut results = Vec::new();
                for (game_id, client) in provider_clients {
                    results.push(operation(**game_id, client.as_ref()).await);
                }
                results
            }
        })
        .collect();

    let all_results = join_all(provider_futures).await;
    collect_results(all_results)
}

/// Collects results from provider batches into a single map, logging failures.
fn collect_results(all_results: Vec<Vec<OperationResult>>) -> HashMap<GameId, serde_json::Value> {
    let mut map = HashMap::new();
    for provider_results in all_results {
        for (game_id, result) in provider_results {
            match result {
                Ok(data) => {
                    tracing::debug!(game_id = ?game_id, "Batch operation succeeded");
                    map.insert(game_id, data);
                }
                Err(e) => {
                    tracing::warn!(game_id = ?game_id, error = %e, "Batch operation failed");
                }
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    type BoxError = Box<dyn std::error::Error + Send + Sync>;
    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    trait MockClient: Send + Sync {
        fn do_work(&self) -> BoxFuture<'_, Result<serde_json::Value, BoxError>>;
    }

    struct TestClient {
        id: GameId,
        should_fail: bool,
    }

    impl MockClient for TestClient {
        fn do_work(&self) -> BoxFuture<'_, Result<serde_json::Value, BoxError>> {
            Box::pin(async {
                if self.should_fail {
                    Err("test error".into())
                } else {
                    Ok(serde_json::json!({"game": self.id.as_str()}))
                }
            })
        }
    }

    fn make_clients(specs: &[(GameId, bool)]) -> HashMap<GameId, Box<dyn MockClient>> {
        specs
            .iter()
            .map(|(id, fail)| {
                (
                    *id,
                    Box::new(TestClient {
                        id: *id,
                        should_fail: *fail,
                    }) as Box<dyn MockClient>,
                )
            })
            .collect()
    }

    #[tokio::test(start_paused = true)]
    async fn empty_clients() {
        let clients: HashMap<GameId, Box<dyn MockClient>> = HashMap::new();
        let result = batch_by_provider(&clients, None, |id, c| {
            Box::pin(async move { (id, c.do_work().await) })
        })
        .await;
        assert!(result.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn all_succeed() {
        let clients = make_clients(&[
            (GameId::GenshinImpact, false),
            (GameId::HonkaiStarRail, false),
            (GameId::WutheringWaves, false),
        ]);
        let result = batch_by_provider(&clients, None, |id, c| {
            Box::pin(async move { (id, c.do_work().await) })
        })
        .await;
        assert_eq!(result.len(), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn partial_failure() {
        let clients = make_clients(&[
            (GameId::GenshinImpact, false),
            (GameId::HonkaiStarRail, true),
        ]);
        let result = batch_by_provider(&clients, None, |id, c| {
            Box::pin(async move { (id, c.do_work().await) })
        })
        .await;
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&GameId::GenshinImpact));
    }

    #[tokio::test(start_paused = true)]
    async fn closure_can_add_side_effects() {
        let clients = make_clients(&[(GameId::GenshinImpact, false)]);
        let counter = AtomicU32::new(0);
        let result = batch_by_provider(&clients, None, |id, c| {
            let counter = &counter;
            Box::pin(async move {
                counter.fetch_add(1, Ordering::Relaxed);
                (id, c.do_work().await)
            })
        })
        .await;
        assert_eq!(result.len(), 1);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }
}
