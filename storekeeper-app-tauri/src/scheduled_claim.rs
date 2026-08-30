//! Scheduled auto-claim for daily rewards.

use crate::events::AppEvent;
use crate::state::AppState;
use anyhow::Context;
use jiff::SignedDuration;
use jiff::Timestamp;
use std::collections::HashMap;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::Duration;
use storekeeper_client_hoyolab::Error as HoyolabError;
use storekeeper_core::ClaimResult;
use storekeeper_core::ClaimTime;
use storekeeper_core::DailyRewardStatus;
use storekeeper_core::GameId;
use storekeeper_core::next_claim_datetime_utc;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

/// Maximum chunk duration for wall-clock-bounded sleeps.
///
/// Prevents long `tokio::time::sleep` calls that can drift during OS suspend.
const MAX_SLEEP_CHUNK: Duration = Duration::from_mins(15);

/// Short sleep used when no games are configured or no claims are pending.
const IDLE_SLEEP: Duration = Duration::from_mins(15);

/// Why the scheduler woke up from a sleep.
enum WakeReason {
    /// The cancellation token was triggered (app shutdown).
    Cancelled,
    /// Config changed - should re-read state and re-run startup claims.
    ConfigChanged,
    /// The timer expired normally.
    TimerExpired,
}

/// What the scheduler should do after a non-terminal wake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PostWake {
    /// Re-run startup claims before resuming the loop (config changed).
    Rerun,
    /// Resume the loop normally (timer expired).
    Resume,
}

/// Result of a scheduled claim attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClaimOutcome {
    /// The attempt registered the reward.
    Claimed,
    /// The API reported the reward already signed before this attempt.
    AlreadyClaimed,
}

const CLAIM_RETRY_DELAY: SignedDuration = SignedDuration::from_mins(10);

const MAX_CLAIM_RETRIES: u32 = 3;

fn is_recoverable(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<HoyolabError>()
        .is_none_or(HoyolabError::is_recoverable)
}

#[derive(Debug, Default)]
struct RetryQueue {
    attempts: HashMap<GameId, u32>,
    due: Option<Timestamp>,
}

#[derive(Debug, Default)]
struct ClaimRound {
    attempted: Vec<GameId>,
    failed: Vec<GameId>,
}

impl RetryQueue {
    fn record(&mut self, attempted: &[GameId], failed: &[GameId], now: Timestamp) {
        for game_id in attempted {
            let spent = failed
                .contains(game_id)
                .then(|| self.attempts.get(game_id).copied().unwrap_or(0) + 1)
                .filter(|spent| *spent <= MAX_CLAIM_RETRIES);

            match spent {
                Some(spent) => self.attempts.insert(*game_id, spent),
                None => self.attempts.remove(game_id),
            };
        }

        // If adding the retry delay fails, leave `due` unchanged; an immediate
        // fallback would consume every attempt.
        if !failed.is_empty()
            && let Ok(next) = now.saturating_add(CLAIM_RETRY_DELAY)
        {
            self.due = Some(self.due.map_or(next, |due| due.min(next)));
        }

        if self.attempts.is_empty() {
            self.due = None;
        }
    }

    fn clear_games(&mut self, games: &[GameId]) {
        for game_id in games {
            self.attempts.remove(game_id);
        }

        if self.attempts.is_empty() {
            self.due = None;
        }
    }

    fn pending(&self) -> Option<(Timestamp, Vec<GameId>)> {
        let due = self.due?;
        Some((due, self.attempts.keys().copied().collect()))
    }
}

#[derive(Debug, PartialEq, Eq)]
enum WakeKind {
    Daily,
    Retry,
}

#[derive(Debug, PartialEq, Eq)]
struct ClaimWake {
    kind: WakeKind,
    target: Timestamp,
    games: Vec<GameId>,
}

fn select_wake(
    daily: Option<(Timestamp, Vec<GameId>)>,
    retry: Option<(Timestamp, Vec<GameId>)>,
) -> Option<ClaimWake> {
    let daily = daily.map(|(target, games)| ClaimWake {
        kind: WakeKind::Daily,
        target,
        games,
    });
    let retry = retry.map(|(target, games)| ClaimWake {
        kind: WakeKind::Retry,
        target,
        games,
    });

    match (daily, retry) {
        (Some(daily), Some(retry)) if retry.target < daily.target => Some(retry),
        (Some(daily), _) => Some(daily),
        (None, retry) => retry,
    }
}

/// Require the claim response to report a recorded reward.
///
/// Use `info.is_signed` rather than `success`: a registered claim can lack
/// reward details, while an unsigned response did not record the claim.
fn ensure_claim_registered(result: &ClaimResult) -> anyhow::Result<()> {
    anyhow::ensure!(
        result.info.is_signed,
        "claim did not register: {}",
        result.message.as_deref().unwrap_or("no message")
    );
    Ok(())
}

/// Pure mapping from a wake reason to the loop's next control-flow step.
///
/// `Break(())` means the scheduler should stop (cancellation). `Continue(_)`
/// means keep looping, with the [`PostWake`] payload describing what
/// bookkeeping the caller should perform first. Kept side-effect-free so it is
/// unit-testable for every [`WakeReason`] variant.
fn handle_wake_reason(reason: &WakeReason) -> ControlFlow<(), PostWake> {
    match reason {
        WakeReason::Cancelled => ControlFlow::Break(()),
        WakeReason::ConfigChanged => ControlFlow::Continue(PostWake::Rerun),
        WakeReason::TimerExpired => ControlFlow::Continue(PostWake::Resume),
    }
}

/// Starts the scheduled daily claim task.
///
/// This spawns a tokio task that:
/// 1. Runs startup claims for any games that haven't been claimed today
/// 2. Enters the main scheduling loop to claim at configured times
///
/// The scheduler wakes on three events:
/// - Timer expiry (wall-clock-bounded in 15 min chunks to survive OS suspend)
/// - Config change notification via `AppState::wake_scheduler()`
/// - Cancellation token (app shutdown)
pub fn start_scheduled_claims(app_handle: AppHandle, cancel_token: CancellationToken) {
    tauri::async_runtime::spawn(async move {
        tracing::info!("Starting scheduled daily reward claim task");

        let state = app_handle.state::<AppState>();
        let notify = state.scheduler_notify();

        let mut retry_queue = RetryQueue::default();
        let round = run_startup_claims(&state, &app_handle).await;
        retry_queue.record(&round.attempted, &round.failed, Timestamp::now());

        loop {
            let auto_claim_games = state.get_auto_claim_games().await;
            let daily = calculate_next_claim(&auto_claim_games, &state).await;

            let Some(wake) = select_wake(daily, retry_queue.pending()) else {
                tracing::debug!("No games need claiming, idle sleeping");
                match idle_wait(&cancel_token, &notify, &state, &app_handle).await {
                    ControlFlow::Break(()) => break,
                    ControlFlow::Continue(round) => {
                        retry_queue.record(&round.attempted, &round.failed, Timestamp::now());
                        continue;
                    }
                }
            };

            // Clamp to zero for display: a target in the past sleeps for no
            // time.
            let until_claim_secs = wake
                .target
                .duration_since(Timestamp::now())
                .as_secs()
                .max(0);

            tracing::info!(
                sleep_secs = until_claim_secs,
                target = %wake.target,
                games = ?wake.games,
                kind = ?wake.kind,
                "Waiting until next scheduled claim time"
            );

            match handle_wake_reason(&sleep_until(wake.target, &cancel_token, &notify).await) {
                ControlFlow::Break(()) => break,
                ControlFlow::Continue(PostWake::Rerun) => {
                    tracing::info!(
                        "Config changed while waiting for claim, re-running startup claims"
                    );
                    let round = run_startup_claims(&state, &app_handle).await;
                    retry_queue.record(&round.attempted, &round.failed, Timestamp::now());
                }
                ControlFlow::Continue(PostWake::Resume) => {
                    if wake.kind == WakeKind::Daily {
                        retry_queue.clear_games(&wake.games);
                    }
                    let round = claim_games_and_emit(&state, &app_handle, &wake.games).await;
                    retry_queue.record(&round.attempted, &round.failed, Timestamp::now());
                }
            }
        }
    });
}

/// Sleeps idly and processes the resulting wake.
///
/// Returns [`ControlFlow::Break`] when the scheduler should stop, and
/// [`ControlFlow::Continue`] when the loop should iterate again (re-running
/// startup claims first if the config changed).
async fn idle_wait(
    cancel_token: &CancellationToken,
    notify: &Arc<Notify>,
    state: &AppState,
    app_handle: &AppHandle,
) -> ControlFlow<(), ClaimRound> {
    match handle_wake_reason(&sleep_short(cancel_token, notify).await) {
        ControlFlow::Break(()) => ControlFlow::Break(()),
        ControlFlow::Continue(post) => {
            if post == PostWake::Rerun {
                tracing::info!("Config changed while idle, re-running startup claims");
                return ControlFlow::Continue(run_startup_claims(state, app_handle).await);
            }
            ControlFlow::Continue(ClaimRound::default())
        }
    }
}

/// Runs startup claims for games that have auto-claim enabled.
///
/// For each game, checks the API status first - if not claimed today,
/// attempts to claim.
async fn run_startup_claims(state: &AppState, app_handle: &AppHandle) -> ClaimRound {
    tracing::info!("Running startup auto-claim check");

    let auto_claim_games = state.get_auto_claim_games().await;

    if auto_claim_games.is_empty() {
        tracing::debug!("No games with auto-claim enabled");
        return ClaimRound::default();
    }

    let game_ids: Vec<GameId> = auto_claim_games.into_iter().map(|(id, _)| id).collect();
    claim_games_and_emit(state, app_handle, &game_ids).await
}

/// Claims rewards for the given games and emits results to the frontend.
async fn claim_games_and_emit(
    state: &AppState,
    app_handle: &AppHandle,
    game_ids: &[GameId],
) -> ClaimRound {
    let mut claimed = Vec::new();
    let mut failed = Vec::new();

    for &game_id in game_ids {
        if !state.should_auto_claim_game(game_id).await {
            tracing::debug!(game_id = ?game_id, "Skipping auto-claim (disabled in config)");
            continue;
        }

        tracing::info!(game_id = ?game_id, "Auto-claiming daily reward");

        match claim_with_status_check(state, game_id).await {
            Ok(ClaimOutcome::Claimed) => {
                tracing::info!(game_id = ?game_id, "Auto-claim successful");
                claimed.push(game_id);
            }
            Ok(ClaimOutcome::AlreadyClaimed) => {
                tracing::debug!(game_id = ?game_id, "Already claimed today (per API)");
            }
            Err(e) => {
                let recoverable = is_recoverable(&e);
                tracing::error!(
                    game_id = ?game_id,
                    error = %crate::logging::error_chain(&*e),
                    recoverable = recoverable,
                    "Auto-claim failed"
                );
                if recoverable {
                    failed.push(game_id);
                }
            }
        }

        // Small delay between games to avoid rate limiting
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    if !claimed.is_empty() {
        let status = state.fetch_all_daily_reward_status().await;
        let results: HashMap<GameId, serde_json::Value> = claimed
            .iter()
            .filter_map(|game_id| {
                status
                    .games
                    .get(game_id)
                    .map(|value| (*game_id, value.clone()))
            })
            .collect();
        state.set_daily_reward_status(status).await;

        if let Err(e) = app_handle.emit(AppEvent::DailyRewardClaimed.as_str(), &results) {
            tracing::warn!(error = %e, "Failed to emit daily reward claimed event");
        }

        tracing::info!(games_claimed = claimed.len(), "Auto-claim complete");
    }

    ClaimRound {
        attempted: game_ids.to_vec(),
        failed,
    }
}

/// Sleeps until the given wall-clock target, in bounded chunks.
///
/// Sleeps in chunks of at most [`MAX_SLEEP_CHUNK`] and re-checks
/// `Timestamp::now()` after each chunk. This ensures the scheduler fires
/// promptly after OS suspend/resume, with at most one chunk of delay.
async fn sleep_until(
    target: Timestamp,
    cancel_token: &CancellationToken,
    notify: &Arc<Notify>,
) -> WakeReason {
    loop {
        let now = Timestamp::now();
        if now >= target {
            return WakeReason::TimerExpired;
        }

        // `now < target` here, so the duration is positive; `unsigned_abs`
        // yields the std `Duration` tokio needs.
        let remaining = target.duration_since(now).unsigned_abs();
        let chunk = remaining.min(MAX_SLEEP_CHUNK);

        tokio::select! {
            () = cancel_token.cancelled() => {
                tracing::info!("Scheduled claims cancelled");
                return WakeReason::Cancelled;
            }
            () = notify.notified() => {
                return WakeReason::ConfigChanged;
            }
            () = tokio::time::sleep(chunk) => {
                // Re-check the wall clock; a config change may have moved it.
            }
        }
    }
}

/// Short idle sleep with cancel/notify support.
///
/// Used when no games are configured or no claims are pending.
async fn sleep_short(cancel_token: &CancellationToken, notify: &Arc<Notify>) -> WakeReason {
    tokio::select! {
        () = cancel_token.cancelled() => {
            tracing::info!("Scheduled claims cancelled");
            WakeReason::Cancelled
        }
        () = notify.notified() => {
            WakeReason::ConfigChanged
        }
        () = tokio::time::sleep(IDLE_SLEEP) => {
            WakeReason::TimerExpired
        }
    }
}

/// Checks status and claims if not already claimed today.
///
/// # Errors
///
/// Returns an error when status or claim retrieval fails, or when the response
/// reports the reward still unsigned.
async fn claim_with_status_check(
    state: &AppState,
    game_id: GameId,
) -> anyhow::Result<ClaimOutcome> {
    let status = state.get_daily_reward_status_for_game(game_id).await?;

    let reward_status: DailyRewardStatus =
        serde_json::from_value(status).context("failed to deserialize daily reward status")?;

    if reward_status.info.is_signed {
        return Ok(ClaimOutcome::AlreadyClaimed);
    }

    let claimed = state.claim_daily_reward_for_game(game_id).await?;
    let result: ClaimResult =
        serde_json::from_value(claimed).context("failed to deserialize claim result")?;

    ensure_claim_registered(&result)?;

    Ok(ClaimOutcome::Claimed)
}

/// Calculates the next claim time and which games to claim.
///
/// Returns the target wall-clock datetime and the list of games to claim at
/// that time. Returns `None` if no games need claiming.
async fn calculate_next_claim(
    auto_claim_games: &[(GameId, Option<ClaimTime>)],
    state: &AppState,
) -> Option<(Timestamp, Vec<GameId>)> {
    let mut earliest_time = None;
    let mut games_at_earliest: Vec<GameId> = Vec::new();

    for (game_id, claim_time) in auto_claim_games {
        if !state.should_auto_claim_game(*game_id).await {
            continue;
        }

        let next_claim = match next_claim_datetime_utc(*claim_time) {
            Ok(dt) => dt,
            Err(e) => {
                tracing::error!(
                    game_id = ?game_id,
                    error = %e,
                    "Failed to calculate next claim time"
                );
                continue;
            }
        };

        match earliest_time {
            None => {
                earliest_time = Some(next_claim);
                games_at_earliest = vec![*game_id];
            }
            Some(earliest) => {
                if next_claim < earliest {
                    earliest_time = Some(next_claim);
                    games_at_earliest = vec![*game_id];
                } else if next_claim == earliest {
                    games_at_earliest.push(*game_id);
                }
            }
        }
    }

    let earliest = earliest_time?;

    // If the target is in the past, `sleep_until` will return immediately
    Some((earliest, games_at_earliest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use storekeeper_core::DailyReward;
    use storekeeper_core::DailyRewardInfo;

    fn reward() -> DailyReward {
        DailyReward::new("Primogems", 60, "primogem.png")
    }

    #[test]
    fn ensure_claim_registered_accepts_a_signed_reward() {
        let result = ClaimResult::success(reward(), DailyRewardInfo::new(true, 1));

        assert!(
            ensure_claim_registered(&result).is_ok(),
            "a signed reward is a registered claim"
        );
    }

    #[test]
    fn ensure_claim_registered_accepts_a_claim_whose_reward_name_is_missing() {
        let result = ClaimResult::error(
            "Claim succeeded but reward details unavailable",
            DailyRewardInfo::new(true, 1),
        );

        assert!(
            ensure_claim_registered(&result).is_ok(),
            "an unnamed reward must still emit and refresh, not read as a prior claim"
        );
    }

    #[test]
    fn ensure_claim_registered_rejects_success_over_an_unsigned_reward() {
        let result = ClaimResult::success(reward(), DailyRewardInfo::new(false, 0));

        assert!(
            ensure_claim_registered(&result).is_err(),
            "a success flag over an unsigned reward must not read as a claim"
        );
    }

    #[test]
    fn ensure_claim_registered_rejects_an_error_result() {
        let result = ClaimResult::error("boom", DailyRewardInfo::new(false, 0));

        assert!(
            ensure_claim_registered(&result).is_err(),
            "an error result must not read as a claim"
        );
    }

    #[test]
    fn ensure_claim_registered_rejects_a_prior_claim_that_reads_unsigned() {
        let result = ClaimResult::already_claimed(Some(reward()), DailyRewardInfo::new(false, 0));

        assert!(
            ensure_claim_registered(&result).is_err(),
            "a contradictory response must not read as a claim"
        );
    }

    #[test]
    fn handle_wake_reason_cancelled_breaks() {
        assert_eq!(
            handle_wake_reason(&WakeReason::Cancelled),
            ControlFlow::Break(())
        );
    }

    #[test]
    fn handle_wake_reason_config_changed_reruns_startup() {
        assert_eq!(
            handle_wake_reason(&WakeReason::ConfigChanged),
            ControlFlow::Continue(PostWake::Rerun)
        );
    }

    #[test]
    fn handle_wake_reason_timer_expired_resumes() {
        assert_eq!(
            handle_wake_reason(&WakeReason::TimerExpired),
            ControlFlow::Continue(PostWake::Resume)
        );
    }

    fn registry_error(cause: HoyolabError) -> anyhow::Error {
        let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(cause);
        crate::daily_reward_registry::into_anyhow(boxed)
            .context("failed to fetch daily reward status")
    }

    #[test]
    fn is_recoverable_reads_a_throttle_through_the_boxed_chain() {
        let error = registry_error(HoyolabError::RateLimited {
            retcode: -1004,
            message: "Too many attempts. Please try again later.".to_string(),
        });

        assert!(is_recoverable(&error), "a wrapped throttle is recoverable");
    }

    #[test]
    fn is_recoverable_rejects_a_bad_cookie_through_the_boxed_chain() {
        let error = registry_error(HoyolabError::Client(
            storekeeper_client_hoyolab::ClientError::api_error(-100, "not logged in"),
        ));

        assert!(
            !is_recoverable(&error),
            "a wrapped cookie error is not recoverable"
        );
    }

    #[test]
    fn is_recoverable_requeues_a_failure_carrying_no_client_error() {
        let error = anyhow::anyhow!("the scheduler could not reach the registry");

        assert!(
            is_recoverable(&error),
            "a failure without a HoYoLab error is recoverable"
        );
    }

    fn instant(rfc3339: &str) -> Timestamp {
        rfc3339.parse().expect("a valid RFC 3339 instant")
    }

    const GENSHIN: GameId = GameId::GenshinImpact;
    const HSR: GameId = GameId::HonkaiStarRail;

    #[test]
    fn record_schedules_the_failed_games_after_the_retry_delay() {
        let now = instant("2026-08-27T22:00:10Z");
        let mut queue = RetryQueue::default();

        queue.record(&[GENSHIN, HSR], &[GENSHIN], now);

        let (due, games) = queue.pending().expect("a recorded failure is pending");
        assert_eq!(due, instant("2026-08-27T22:10:10Z"));
        assert_eq!(games, vec![GENSHIN]);
    }

    #[test]
    fn record_drops_a_game_that_spent_every_retry() {
        let mut now = instant("2026-08-27T22:00:10Z");
        let mut queue = RetryQueue::default();

        for _ in 0..=MAX_CLAIM_RETRIES {
            queue.record(&[GENSHIN], &[GENSHIN], now);
            now = now
                .saturating_add(CLAIM_RETRY_DELAY)
                .expect("the retry delay stays in range");
        }

        assert!(
            queue.pending().is_none(),
            "a game must stop retrying after {MAX_CLAIM_RETRIES} attempts"
        );
    }

    #[test]
    fn record_leaves_a_game_the_round_did_not_attempt() {
        let now = instant("2026-08-27T22:00:10Z");
        let mut queue = RetryQueue::default();
        queue.record(&[GENSHIN], &[GENSHIN], now);

        queue.record(&[HSR], &[], now);

        let (_, games) = queue
            .pending()
            .expect("a game outside the round stays queued");
        assert_eq!(games, vec![GENSHIN]);
    }

    #[test]
    fn record_keeps_the_earliest_due_instant() {
        let first = instant("2026-08-27T22:00:10Z");
        let mut queue = RetryQueue::default();
        queue.record(&[GENSHIN], &[GENSHIN], first);

        queue.record(&[HSR], &[HSR], instant("2026-08-27T22:05:10Z"));

        let (due, _) = queue.pending().expect("both games are queued");
        assert_eq!(
            due,
            instant("2026-08-27T22:10:10Z"),
            "a later failure must not push an already-queued game back"
        );
    }

    #[test]
    fn record_drops_an_attempted_game_that_stopped_failing() {
        let now = instant("2026-08-27T22:00:10Z");
        let mut queue = RetryQueue::default();

        queue.record(&[GENSHIN, HSR], &[GENSHIN, HSR], now);
        queue.record(&[GENSHIN, HSR], &[GENSHIN], now);

        let (_, games) = queue.pending().expect("the still-failing game is pending");
        assert_eq!(games, vec![GENSHIN]);
    }

    #[test]
    fn clear_games_restores_the_full_retry_budget() {
        let now = instant("2026-08-27T22:00:10Z");
        let mut queue = RetryQueue::default();
        for _ in 0..MAX_CLAIM_RETRIES {
            queue.record(&[GENSHIN], &[GENSHIN], now);
        }

        queue.clear_games(&[GENSHIN]);
        for _ in 0..MAX_CLAIM_RETRIES {
            queue.record(&[GENSHIN], &[GENSHIN], now);
        }

        assert!(
            queue.pending().is_some(),
            "a daily claim must restore the game's full retry budget"
        );
    }

    #[test]
    fn clear_games_leaves_a_game_the_daily_claim_does_not_cover() {
        let now = instant("2026-08-27T22:00:10Z");
        let mut queue = RetryQueue::default();
        queue.record(&[GENSHIN, HSR], &[GENSHIN, HSR], now);

        queue.clear_games(&[HSR]);

        let (_, games) = queue
            .pending()
            .expect("a game outside the cleared set stays queued");
        assert_eq!(games, vec![GENSHIN]);
    }

    #[test]
    fn select_wake_prefers_whichever_claim_comes_first() {
        let early = instant("2026-08-27T22:10:00Z");
        let late = instant("2026-08-28T16:00:00Z");
        let daily_games = vec![HSR];
        let retry_games = vec![GENSHIN];

        for (daily_at, retry_at, expected_kind, expected_target) in [
            (Some(late), Some(early), Some(WakeKind::Retry), Some(early)),
            (Some(early), Some(late), Some(WakeKind::Daily), Some(early)),
            (Some(late), None, Some(WakeKind::Daily), Some(late)),
            (None, Some(early), Some(WakeKind::Retry), Some(early)),
            (None, None, None, None),
        ] {
            let selected = select_wake(
                daily_at.map(|target| (target, daily_games.clone())),
                retry_at.map(|target| (target, retry_games.clone())),
            );

            assert_eq!(
                selected.as_ref().map(|wake| &wake.kind),
                expected_kind.as_ref(),
                "daily at {daily_at:?} against retry at {retry_at:?}"
            );
            assert_eq!(
                selected.map(|wake| wake.target),
                expected_target,
                "daily at {daily_at:?} against retry at {retry_at:?}"
            );
        }
    }

    #[test]
    fn select_wake_breaks_a_tie_in_favour_of_the_daily_claim() {
        let target = instant("2026-08-27T22:10:00Z");

        let selected = select_wake(Some((target, vec![HSR])), Some((target, vec![GENSHIN])))
            .expect("a tie selects the daily claim");

        assert_eq!(selected.kind, WakeKind::Daily);
        assert_eq!(selected.games, vec![HSR]);
    }
}
