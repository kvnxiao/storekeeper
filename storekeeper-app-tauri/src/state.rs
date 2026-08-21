//! Application state management.

use crate::clients::create_daily_reward_registry;
use crate::clients::create_registry;
use crate::daily_reward_registry::DailyRewardRegistry;
use crate::notification::NotificationTracker;
use crate::registry::GameClientRegistry;
use jiff::Timestamp;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use storekeeper_core::AppConfig;
use storekeeper_core::ClaimTime;
use storekeeper_core::GameId;
use storekeeper_core::SecretsConfig;
use storekeeper_core::ensure_configs_exist;
use tokio::sync::Notify;
use tokio::sync::RwLock;

/// All resources from all games.
///
/// Resources are stored as a map from game ID to JSON value, allowing
/// for dynamic game support without explicit per-game fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllResources {
    /// Resources keyed by game ID.
    ///
    /// Each value is a JSON array of the game's resource types.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub games: HashMap<GameId, serde_json::Value>,

    /// Last update timestamp.
    pub last_updated: Option<Timestamp>,
}

/// All daily reward status from all games.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllDailyRewardStatus {
    /// Reward status keyed by game ID.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub games: HashMap<GameId, serde_json::Value>,

    /// Last check timestamp.
    pub last_checked: Option<Timestamp>,
}

/// Merges freshly fetched games over the previously cached ones.
///
/// A game whose fetch failed is absent from `fetched` and keeps its last known
/// values, so one failing provider does not blank its cards. A game that is no
/// longer configured is dropped instead of lingering as a resource nothing
/// refreshes.
fn merge_game_resources(
    previous: HashMap<GameId, serde_json::Value>,
    fetched: HashMap<GameId, serde_json::Value>,
    configured: &HashSet<GameId>,
) -> HashMap<GameId, serde_json::Value> {
    let mut games: HashMap<GameId, serde_json::Value> = previous
        .into_iter()
        .filter(|(game_id, _)| configured.contains(game_id))
        .collect();
    games.extend(fetched);
    games
}

/// Inner state data protected by `RwLock`.
#[derive(Default)]
pub struct StateData {
    /// Cached resources from all games.
    pub resources: AllResources,

    /// Registry-based game clients.
    pub registry: Arc<GameClientRegistry>,

    /// Daily reward client registry.
    pub daily_reward_registry: Arc<DailyRewardRegistry>,

    /// Cached daily reward status.
    pub daily_reward_status: AllDailyRewardStatus,

    /// Application configuration.
    pub config: AppConfig,

    /// Secrets configuration (kept in memory for diff detection on reload).
    pub secrets: SecretsConfig,

    /// Notification cooldown tracker.
    pub notification_tracker: NotificationTracker,
}

/// Application state wrapper.
#[derive(Clone)]
pub struct AppState {
    /// Inner state protected by async `RwLock`.
    pub inner: Arc<RwLock<StateData>>,
    refreshing: Arc<AtomicBool>,
    /// Notifier to wake the scheduler when config changes.
    scheduler_notify: Arc<Notify>,
}

impl AppState {
    /// Creates a new application state with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(StateData::default())),
            refreshing: Arc::new(AtomicBool::new(false)),
            scheduler_notify: Arc::new(Notify::new()),
        }
    }

    /// Creates a new application state with initialized clients from config.
    ///
    /// Attempts to load configuration and secrets files. If they don't exist,
    /// creates default config files first, then loads them.
    #[must_use]
    pub fn with_config() -> Self {
        if let Err(e) = ensure_configs_exist() {
            tracing::warn!("Failed to ensure config files exist: {e}");
        }

        let config = AppConfig::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load config, using defaults: {e}");
            AppConfig::default()
        });
        let secrets = SecretsConfig::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load secrets, using defaults: {e}");
            SecretsConfig::default()
        });

        let registry = create_registry(&config, &secrets);
        let daily_reward_registry = create_daily_reward_registry(&config, &secrets);

        Self {
            inner: Arc::new(RwLock::new(StateData {
                resources: AllResources::default(),
                registry: Arc::new(registry),
                daily_reward_registry: Arc::new(daily_reward_registry),
                daily_reward_status: AllDailyRewardStatus::default(),
                config,
                secrets,
                notification_tracker: NotificationTracker::default(),
            })),
            refreshing: Arc::new(AtomicBool::new(false)),
            scheduler_notify: Arc::new(Notify::new()),
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

    /// Attempts to mark refresh as started.
    ///
    /// Returns `true` if this call acquired the refresh slot.
    #[must_use]
    pub fn try_start_refresh(&self) -> bool {
        self.refreshing
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Marks refresh as finished.
    pub fn finish_refresh(&self) {
        self.refreshing.store(false, Ordering::Release);
    }

    /// Wakes the scheduler to re-evaluate config (e.g. after auto-claim
    /// changes).
    pub fn wake_scheduler(&self) {
        self.scheduler_notify.notify_one();
    }

    /// Returns the scheduler notify handle for use in the scheduler task.
    #[must_use]
    pub fn scheduler_notify(&self) -> Arc<Notify> {
        Arc::clone(&self.scheduler_notify)
    }

    /// Fetches every configured game client and merges the results into the
    /// cached resources.
    ///
    /// Emits per-game update events via the app handle as each game completes.
    pub async fn refresh_all_resources(&self, app_handle: &tauri::AppHandle) -> AllResources {
        let registry = {
            let state = self.inner.read().await;
            Arc::clone(&state.registry)
        };
        let fetched = registry.fetch_all(app_handle).await;

        let mut state = self.inner.write().await;
        let previous = std::mem::take(&mut state.resources.games);
        state.resources = AllResources {
            games: merge_game_resources(previous, fetched, &registry.game_ids()),
            last_updated: Some(Timestamp::now()),
        };
        state.resources.clone()
    }

    /// Returns the poll interval from config.
    pub async fn poll_interval_secs(&self) -> u64 {
        let state = self.inner.read().await;
        state.config.general.poll_interval_secs
    }

    /// Returns whether any game clients are configured.
    pub async fn has_clients(&self) -> bool {
        let registry = {
            let state = self.inner.read().await;
            Arc::clone(&state.registry)
        };
        registry.has_any()
    }

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
        let daily_reward_registry = {
            let state = self.inner.read().await;
            Arc::clone(&state.daily_reward_registry)
        };
        let games = daily_reward_registry.get_all_status().await;
        AllDailyRewardStatus {
            games,
            last_checked: Some(Timestamp::now()),
        }
    }

    /// Re-fetches the games that have a daily-reward client but no cached
    /// status, and merges what comes back into the cache.
    ///
    /// A game drops out of the cache when its fetch fails, and the fetch of
    /// every game only runs at startup, at the daily reset, and after a claim.
    /// Without this, one failed request hides that game's claim badge until the
    /// next of those. Returns whether the cache changed.
    pub async fn refill_missing_daily_reward_status(&self) -> bool {
        let (daily_reward_registry, missing) = {
            let state = self.inner.read().await;
            let missing: HashSet<GameId> = state
                .daily_reward_registry
                .game_ids()
                .into_iter()
                .filter(|game_id| !state.daily_reward_status.games.contains_key(game_id))
                .collect();
            (Arc::clone(&state.daily_reward_registry), missing)
        };

        if missing.is_empty() {
            return false;
        }

        tracing::debug!(games = ?missing, "Re-fetching daily reward status missing from cache");
        let fetched = daily_reward_registry.get_status_for_games(&missing).await;
        if fetched.is_empty() {
            return false;
        }

        let mut state = self.inner.write().await;
        let cached = &mut state.daily_reward_status.games;
        // A claim can land while the fetch is in flight, so fill only the holes
        // that are still holes rather than overwriting a fresher status.
        let mut filled = false;
        for (game_id, status) in fetched {
            if let Entry::Vacant(slot) = cached.entry(game_id) {
                slot.insert(status);
                filled = true;
            }
        }

        if filled {
            state.daily_reward_status.last_checked = Some(Timestamp::now());
        }
        filled
    }

    /// Claims daily reward for a specific game.
    ///
    /// # Errors
    ///
    /// Returns an error if the game is not configured or the claim fails.
    pub async fn claim_daily_reward_for_game(
        &self,
        game_id: GameId,
    ) -> anyhow::Result<serde_json::Value> {
        let daily_reward_registry = {
            let state = self.inner.read().await;
            Arc::clone(&state.daily_reward_registry)
        };
        daily_reward_registry.claim_for_game(game_id).await
    }

    /// Gets the daily reward status for a specific game.
    ///
    /// # Errors
    ///
    /// Returns an error if the game is not configured or the fetch fails.
    pub async fn get_daily_reward_status_for_game(
        &self,
        game_id: GameId,
    ) -> anyhow::Result<serde_json::Value> {
        let daily_reward_registry = {
            let state = self.inner.read().await;
            Arc::clone(&state.daily_reward_registry)
        };
        daily_reward_registry.get_status_for_game(game_id).await
    }

    /// Gets the list of games that have auto-claim enabled.
    ///
    /// Returns a list of `(GameId, Option<ClaimTime>)` pairs.
    pub async fn get_auto_claim_games(&self) -> Vec<(GameId, Option<ClaimTime>)> {
        let state = self.inner.read().await;
        GameId::all()
            .iter()
            .filter(|id| state.config.games.auto_claim_enabled(**id))
            .map(|id| (*id, state.config.games.auto_claim_time(*id)))
            .collect()
    }

    /// Checks if auto-claim is enabled for a specific game.
    ///
    /// Returns true if auto-claim is enabled in config and the game is
    /// registered in the daily reward registry. Does not check if already
    /// claimed today - that is determined by fetching status from the API.
    pub async fn should_auto_claim_game(&self, game_id: GameId) -> bool {
        let state = self.inner.read().await;
        state.config.games.auto_claim_enabled(game_id)
            && state.daily_reward_registry.has_game(game_id)
    }

    /// Fetches resources from a subset of configured game clients.
    pub async fn fetch_resources_for_games(
        &self,
        game_ids: &HashSet<GameId>,
        app_handle: &tauri::AppHandle,
    ) -> HashMap<GameId, serde_json::Value> {
        let registry = {
            let state = self.inner.read().await;
            Arc::clone(&state.registry)
        };
        registry.fetch_for_games(game_ids, app_handle).await
    }

    /// Fetches daily reward status from a subset of configured games.
    pub async fn fetch_daily_reward_status_for_games(
        &self,
        game_ids: &HashSet<GameId>,
    ) -> HashMap<GameId, serde_json::Value> {
        let daily_reward_registry = {
            let state = self.inner.read().await;
            Arc::clone(&state.daily_reward_registry)
        };
        daily_reward_registry.get_status_for_games(game_ids).await
    }

    /// Applies new config and secrets to state, optionally rebuilding
    /// registries.
    ///
    /// When `rebuild_registries` is true, game client and daily reward
    /// registries are recreated from the new config/secrets. This is only
    /// needed when game-level settings (uid, region, enabled) or
    /// credentials change.
    pub async fn apply_config(
        &self,
        config: AppConfig,
        secrets: SecretsConfig,
        rebuild_registries: bool,
    ) {
        if rebuild_registries {
            let registry = create_registry(&config, &secrets);
            let daily_reward_registry = create_daily_reward_registry(&config, &secrets);

            let mut state = self.inner.write().await;
            state.config = config;
            state.secrets = secrets;
            state.registry = Arc::new(registry);
            state.daily_reward_registry = Arc::new(daily_reward_registry);
        } else {
            let mut state = self.inner.write().await;
            state.config = config;
            state.secrets = secrets;
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resources(entries: &[(GameId, &str)]) -> HashMap<GameId, serde_json::Value> {
        entries
            .iter()
            .map(|(game_id, marker)| (*game_id, serde_json::json!([{ "type": marker }])))
            .collect()
    }

    #[test]
    fn merge_keeps_a_game_that_failed_to_fetch() {
        let merged = merge_game_resources(
            resources(&[
                (GameId::GenshinImpact, "cached"),
                (GameId::HonkaiStarRail, "cached"),
            ]),
            resources(&[(GameId::HonkaiStarRail, "fresh")]),
            &HashSet::from([GameId::GenshinImpact, GameId::HonkaiStarRail]),
        );

        assert_eq!(
            merged,
            resources(&[
                (GameId::GenshinImpact, "cached"),
                (GameId::HonkaiStarRail, "fresh"),
            ]),
            "a game missing from the fetch should keep its last known values"
        );
    }

    #[test]
    fn merge_drops_a_game_that_is_no_longer_configured() {
        let merged = merge_game_resources(
            resources(&[
                (GameId::GenshinImpact, "cached"),
                (GameId::HonkaiStarRail, "cached"),
            ]),
            HashMap::new(),
            &HashSet::from([GameId::GenshinImpact]),
        );

        assert_eq!(merged, resources(&[(GameId::GenshinImpact, "cached")]));
    }

    #[test]
    fn merge_adds_a_newly_configured_game() {
        let merged = merge_game_resources(
            HashMap::new(),
            resources(&[(GameId::ZenlessZoneZero, "fresh")]),
            &HashSet::from([GameId::ZenlessZoneZero]),
        );

        assert_eq!(merged, resources(&[(GameId::ZenlessZoneZero, "fresh")]));
    }

    #[test]
    fn all_resources_default_is_empty() {
        let r = AllResources::default();
        assert!(r.games.is_empty());
        assert!(r.last_updated.is_none());
    }

    #[test]
    fn all_resources_serde_roundtrip_empty() {
        let r = AllResources::default();
        let json = serde_json::to_string(&r).expect("serialize");
        let r2: AllResources = serde_json::from_str(&json).expect("deserialize");
        assert!(r2.games.is_empty());
        assert!(r2.last_updated.is_none());
    }

    #[test]
    fn all_resources_serde_roundtrip_with_data() {
        let mut r = AllResources::default();
        r.games
            .insert(GameId::GenshinImpact, serde_json::json!([{"stamina": 160}]));
        r.last_updated = Some(Timestamp::now());

        let json = serde_json::to_string(&r).expect("serialize");
        let r2: AllResources = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(r2.games.len(), 1);
        assert!(r2.last_updated.is_some());
    }

    #[test]
    fn all_resources_camel_case_keys() {
        let r = AllResources {
            last_updated: Some(Timestamp::now()),
            ..AllResources::default()
        };
        let v = serde_json::to_value(&r).expect("serialize");
        assert!(v.get("lastUpdated").is_some(), "should be camelCase");
        assert!(v.get("last_updated").is_none(), "should NOT be snake_case");
    }

    #[test]
    fn all_resources_empty_games_skipped() {
        let r = AllResources::default();
        let v = serde_json::to_value(&r).expect("serialize");
        assert!(
            v.get("games").is_none(),
            "empty games map should be skipped"
        );
    }

    #[test]
    fn all_daily_reward_status_default_is_empty() {
        let s = AllDailyRewardStatus::default();
        assert!(s.games.is_empty());
        assert!(s.last_checked.is_none());
    }

    #[test]
    fn all_daily_reward_status_serde_roundtrip() {
        let mut s = AllDailyRewardStatus::default();
        s.games.insert(
            GameId::HonkaiStarRail,
            serde_json::json!({"is_signed": true}),
        );
        s.last_checked = Some(Timestamp::now());

        let json = serde_json::to_string(&s).expect("serialize");
        let s2: AllDailyRewardStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(s2.games.len(), 1);
        assert!(s2.last_checked.is_some());
    }

    #[test]
    fn all_daily_reward_status_camel_case_keys() {
        let s = AllDailyRewardStatus {
            last_checked: Some(Timestamp::now()),
            ..AllDailyRewardStatus::default()
        };
        let v = serde_json::to_value(&s).expect("serialize");
        assert!(v.get("lastChecked").is_some(), "should be camelCase");
        assert!(v.get("last_checked").is_none(), "should NOT be snake_case");
    }

    struct StubDailyRewardClient {
        game_id: GameId,
        fails: bool,
    }

    impl storekeeper_core::DynDailyRewardClient for StubDailyRewardClient {
        fn game_id(&self) -> GameId {
            self.game_id
        }

        fn get_reward_status_json(
            &self,
        ) -> std::pin::Pin<
            Box<
                dyn Future<
                        Output = Result<
                            serde_json::Value,
                            Box<dyn std::error::Error + Send + Sync>,
                        >,
                    > + Send
                    + '_,
            >,
        > {
            let fails = self.fails;
            Box::pin(async move {
                if fails {
                    Err("stub fetch failure".into())
                } else {
                    Ok(serde_json::json!({"info": {"is_signed": true}}))
                }
            })
        }

        fn claim_daily_reward_json(
            &self,
        ) -> std::pin::Pin<
            Box<
                dyn Future<
                        Output = Result<
                            serde_json::Value,
                            Box<dyn std::error::Error + Send + Sync>,
                        >,
                    > + Send
                    + '_,
            >,
        > {
            Box::pin(async { Ok(serde_json::json!({"success": true})) })
        }
    }

    async fn state_with_daily_reward_clients(clients: &[(GameId, bool)]) -> AppState {
        let mut registry = DailyRewardRegistry::new();
        for &(game_id, fails) in clients {
            registry.register(Box::new(StubDailyRewardClient { game_id, fails }));
        }

        let state = AppState::new();
        state.inner.write().await.daily_reward_registry = Arc::new(registry);
        state
    }

    #[tokio::test]
    async fn refill_fetches_a_game_whose_status_is_absent() {
        let state = state_with_daily_reward_clients(&[
            (GameId::GenshinImpact, false),
            (GameId::HonkaiStarRail, false),
        ])
        .await;
        state
            .set_daily_reward_status(AllDailyRewardStatus {
                games: HashMap::from([(
                    GameId::HonkaiStarRail,
                    serde_json::json!({"info": {"is_signed": true}}),
                )]),
                last_checked: Some(Timestamp::now()),
            })
            .await;

        assert!(state.refill_missing_daily_reward_status().await);

        let status = state.get_daily_reward_status().await;
        assert!(
            status.games.contains_key(&GameId::GenshinImpact),
            "the game missing from the cache should be fetched back in"
        );
        assert!(status.games.contains_key(&GameId::HonkaiStarRail));
    }

    #[tokio::test]
    async fn refill_reports_no_change_when_every_game_is_cached() {
        let state = state_with_daily_reward_clients(&[(GameId::GenshinImpact, false)]).await;
        state
            .set_daily_reward_status(AllDailyRewardStatus {
                games: HashMap::from([(
                    GameId::GenshinImpact,
                    serde_json::json!({"info": {"is_signed": false}}),
                )]),
                last_checked: Some(Timestamp::now()),
            })
            .await;

        assert!(!state.refill_missing_daily_reward_status().await);

        let status = state.get_daily_reward_status().await;
        assert_eq!(
            status.games.get(&GameId::GenshinImpact),
            Some(&serde_json::json!({"info": {"is_signed": false}})),
            "a cached status must not be overwritten by the refill"
        );
    }

    #[tokio::test]
    async fn refill_reports_no_change_when_the_fetch_fails_again() {
        let state = state_with_daily_reward_clients(&[(GameId::GenshinImpact, true)]).await;

        assert!(!state.refill_missing_daily_reward_status().await);
        assert!(state.get_daily_reward_status().await.games.is_empty());
    }
}
