//! Scheduled auto-claim for daily rewards.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use chrono::{DateTime, Utc};
use storekeeper_core::{ClaimTime, DailyRewardStatus, GameId, next_claim_datetime_utc};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::events::AppEvent;
use crate::retry_helpers::retry_with_backoff;
use crate::state::AppState;

/// Maximum chunk duration for wall-clock-bounded sleeps.
///
/// Prevents long `tokio::time::sleep` calls that can drift during OS suspend.
const MAX_SLEEP_CHUNK: Duration = Duration::from_secs(15 * 60);

/// Short sleep used when no games are configured or no claims are pending.
const IDLE_SLEEP: Duration = Duration::from_secs(15 * 60);

/// Why the scheduler woke up from a sleep.
enum WakeReason {
    /// The cancellation token was triggered (app shutdown).
    Cancelled,
    /// Config changed — should re-read state and re-run startup claims.
    ConfigChanged,
    /// The timer expired normally.
    TimerExpired,
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

        // Run startup claims before entering the main loop
        run_startup_claims(&state, &app_handle).await;

        // Main scheduling loop
        loop {
            // Get games that have auto-claim enabled
            let auto_claim_games = state.get_auto_claim_games().await;

            if auto_claim_games.is_empty() {
                tracing::debug!("No games with auto-claim enabled, idle sleeping");
                match sleep_short(&cancel_token, &notify).await {
                    WakeReason::Cancelled => break,
                    WakeReason::ConfigChanged => {
                        tracing::info!("Config changed during idle, re-running startup claims");
                        run_startup_claims(&state, &app_handle).await;
                        continue;
                    }
                    WakeReason::TimerExpired => continue,
                }
            }

            // Find the earliest next claim time across all games
            let Some((target, games_to_claim)) =
                calculate_next_claim(&auto_claim_games, &state).await
            else {
                // No games need claiming right now, idle sleep
                tracing::debug!("No games need claiming, idle sleeping");
                match sleep_short(&cancel_token, &notify).await {
                    WakeReason::Cancelled => break,
                    WakeReason::ConfigChanged => {
                        tracing::info!("Config changed while idle, re-running startup claims");
                        run_startup_claims(&state, &app_handle).await;
                        continue;
                    }
                    WakeReason::TimerExpired => continue,
                }
            };

            let until_claim = (target - Utc::now()).to_std().unwrap_or(Duration::ZERO);

            tracing::info!(
                sleep_secs = until_claim.as_secs(),
                target = %target,
                games = ?games_to_claim,
                "Waiting until next scheduled claim time"
            );

            // Wait until claim time, config change, or cancellation
            match sleep_until(target, &cancel_token, &notify).await {
                WakeReason::Cancelled => break,
                WakeReason::ConfigChanged => {
                    tracing::info!(
                        "Config changed while waiting for claim, re-running startup claims"
                    );
                    run_startup_claims(&state, &app_handle).await;
                }
                WakeReason::TimerExpired => {
                    // Claim rewards for all games that are due
                    claim_games_and_emit(&state, &app_handle, &games_to_claim).await;
                }
            }
        }
    });
}

/// Runs startup claims for games that have auto-claim enabled.
///
/// For each game, checks the API status first - if not claimed today,
/// attempts to claim with retry on network failures.
async fn run_startup_claims(state: &AppState, app_handle: &AppHandle) {
    tracing::info!("Running startup auto-claim check");

    let auto_claim_games = state.get_auto_claim_games().await;

    if auto_claim_games.is_empty() {
        tracing::debug!("No games with auto-claim enabled");
        return;
    }

    let game_ids: Vec<GameId> = auto_claim_games.into_iter().map(|(id, _)| id).collect();
    claim_games_and_emit(state, app_handle, &game_ids).await;
}

/// Claims rewards for the given games and emits results to the frontend.
async fn claim_games_and_emit(state: &AppState, app_handle: &AppHandle, game_ids: &[GameId]) {
    let mut results = HashMap::new();

    for &game_id in game_ids {
        if !state.should_auto_claim_game(game_id).await {
            tracing::debug!(game_id = ?game_id, "Skipping auto-claim (disabled in config)");
            continue;
        }

        tracing::info!(game_id = ?game_id, "Auto-claiming daily reward");

        match claim_with_status_check(state, game_id).await {
            Ok(true) => {
                tracing::info!(game_id = ?game_id, "Auto-claim successful");
                if let Ok(status) = state.get_daily_reward_status_for_game(game_id).await {
                    results.insert(game_id, status);
                }
            }
            Ok(false) => {
                tracing::debug!(game_id = ?game_id, "Already claimed today (per API)");
            }
            Err(e) => {
                tracing::error!(game_id = ?game_id, error = %e, "Auto-claim failed");
            }
        }

        // Small delay between games to avoid rate limiting
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    if !results.is_empty() {
        let status = state.fetch_all_daily_reward_status().await;
        state.set_daily_reward_status(status).await;

        if let Err(e) = app_handle.emit(AppEvent::DailyRewardClaimed.as_str(), &results) {
            tracing::warn!(error = %e, "Failed to emit daily reward claimed event");
        }

        tracing::info!(games_claimed = results.len(), "Auto-claim complete");
    }
}

/// Sleeps until the given wall-clock target, in bounded chunks.
///
/// Sleeps in chunks of at most [`MAX_SLEEP_CHUNK`] and re-checks `Utc::now()`
/// after each chunk. This ensures the scheduler fires promptly after OS
/// suspend/resume, with at most one chunk of delay.
async fn sleep_until(
    target: DateTime<Utc>,
    cancel_token: &CancellationToken,
    notify: &Arc<Notify>,
) -> WakeReason {
    loop {
        let now = Utc::now();
        if now >= target {
            return WakeReason::TimerExpired;
        }

        let remaining = (target - now).to_std().unwrap_or(Duration::ZERO);
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
                // Loop back to re-check wall clock
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
/// Returns `Ok(true)` if claimed, `Ok(false)` if already claimed, `Err` on failure.
async fn claim_with_status_check(state: &AppState, game_id: GameId) -> anyhow::Result<bool> {
    // Step 1: Check status first
    let status = fetch_status_with_retry(state, game_id).await?;

    // Step 2: Check if already claimed via typed deserialization
    let reward_status: DailyRewardStatus =
        serde_json::from_value(status).context("failed to deserialize daily reward status")?;

    if reward_status.info.is_signed {
        return Ok(false); // Already claimed
    }

    // Step 3: Attempt to claim with retry
    claim_reward_with_retry(state, game_id).await?;

    Ok(true)
}

/// Fetches daily reward status with retry on network failures.
async fn fetch_status_with_retry(
    state: &AppState,
    game_id: GameId,
) -> anyhow::Result<serde_json::Value> {
    retry_with_backoff(|| state.get_daily_reward_status_for_game(game_id)).await
}

/// Claims daily reward with retry on network failures.
async fn claim_reward_with_retry(
    state: &AppState,
    game_id: GameId,
) -> anyhow::Result<serde_json::Value> {
    retry_with_backoff(|| state.claim_daily_reward_for_game(game_id)).await
}

/// Calculates the next claim time and which games to claim.
///
/// Returns the target wall-clock datetime and the list of games to claim at
/// that time. Returns `None` if no games need claiming.
async fn calculate_next_claim(
    auto_claim_games: &[(GameId, Option<ClaimTime>)],
    state: &AppState,
) -> Option<(DateTime<Utc>, Vec<GameId>)> {
    let mut earliest_time = None;
    let mut games_at_earliest: Vec<GameId> = Vec::new();

    for (game_id, claim_time) in auto_claim_games {
        // Check if this game has auto-claim enabled
        if !state.should_auto_claim_game(*game_id).await {
            continue;
        }

        // Calculate the next claim time for this game
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
