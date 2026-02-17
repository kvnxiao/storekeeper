//! Scheduled auto-claim for daily rewards.

use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;

use chrono::Utc;
use storekeeper_client_core::retry::RetryConfig;
use storekeeper_core::{ClaimTime, GameId, next_claim_datetime_utc};
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;

use crate::events::AppEvent;
use crate::state::AppState;

/// Starts the scheduled daily claim task.
///
/// This spawns a tokio task that:
/// 1. Runs startup claims for any games that haven't been claimed today
/// 2. Enters the main scheduling loop to claim at configured times
pub fn start_scheduled_claims(app_handle: AppHandle, cancel_token: CancellationToken) {
    tauri::async_runtime::spawn(async move {
        tracing::info!("Starting scheduled daily reward claim task");

        let state = app_handle.state::<AppState>();

        // Run startup claims before entering the main loop
        run_startup_claims(&state, &app_handle).await;

        // Main scheduling loop
        loop {
            // Get games that have auto-claim enabled
            let auto_claim_games = state.get_auto_claim_games().await;

            if auto_claim_games.is_empty() {
                tracing::debug!("No games with auto-claim enabled, sleeping for 60 seconds");
                if sleep_or_cancel(&cancel_token, Duration::from_secs(60)).await {
                    break;
                }
                continue;
            }

            // Find the earliest next claim time across all games
            let Some((sleep_duration, games_to_claim)) =
                calculate_next_claim(&auto_claim_games, &state).await
            else {
                // No games need claiming right now, sleep and retry
                tracing::debug!("No games need claiming, sleeping for 60 seconds");
                if sleep_or_cancel(&cancel_token, Duration::from_secs(60)).await {
                    break;
                }
                continue;
            };

            tracing::info!(
                sleep_secs = sleep_duration.as_secs(),
                games = ?games_to_claim,
                "Waiting until next scheduled claim time"
            );

            // Wait until claim time or cancellation
            if sleep_or_cancel(&cancel_token, sleep_duration).await {
                break;
            }

            // Claim rewards for all games that are due
            claim_games_and_emit(&state, &app_handle, &games_to_claim).await;
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

/// Sleeps for the given duration or until the token is cancelled.
///
/// Returns `true` if cancelled, `false` if the sleep completed.
async fn sleep_or_cancel(cancel_token: &CancellationToken, duration: Duration) -> bool {
    tokio::select! {
        () = cancel_token.cancelled() => {
            tracing::info!("Scheduled claims cancelled");
            true
        }
        () = tokio::time::sleep(duration) => {
            false
        }
    }
}

/// Checks status and claims if not already claimed today.
///
/// Returns `Ok(true)` if claimed, `Ok(false)` if already claimed, `Err` on failure.
async fn claim_with_status_check(state: &AppState, game_id: GameId) -> Result<bool, String> {
    // Step 1: Check status first
    let status = fetch_status_with_retry(state, game_id).await?;

    // Step 2: Check if already claimed
    let is_signed = status
        .get("info")
        .and_then(|i| i.get("is_signed"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if is_signed {
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
) -> Result<serde_json::Value, String> {
    retry_with_backoff(
        || state.get_daily_reward_status_for_game(game_id),
        "fetch status",
        game_id,
    )
    .await
}

/// Claims daily reward with retry on network failures.
async fn claim_reward_with_retry(
    state: &AppState,
    game_id: GameId,
) -> Result<serde_json::Value, String> {
    retry_with_backoff(
        || state.claim_daily_reward_for_game(game_id),
        "claim reward",
        game_id,
    )
    .await
}

/// Generic retry helper using existing `RetryConfig` infrastructure.
async fn retry_with_backoff<F, Fut>(
    mut operation: F,
    operation_name: &str,
    game_id: GameId,
) -> Result<serde_json::Value, String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<serde_json::Value, String>>,
{
    let config = RetryConfig::default(); // 3 retries, 500ms base, 30s max
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if is_retryable_error(&e) && config.should_retry(attempt) => {
                let delay = config.delay_for_attempt(attempt);
                tracing::warn!(
                    game_id = ?game_id,
                    operation = operation_name,
                    attempt = attempt,
                    delay_ms = delay.as_millis(),
                    error = %e,
                    "Request failed, retrying..."
                );
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Determines if an error is retryable (network-related).
fn is_retryable_error(error: &str) -> bool {
    let patterns = [
        "timeout",
        "connection",
        "network",
        "dns",
        "reset",
        "refused",
        "unreachable",
    ];
    let lower = error.to_lowercase();
    patterns.iter().any(|p| lower.contains(p))
}

/// Calculates the next claim time and which games to claim.
///
/// Returns the duration to sleep and the list of games to claim at that time.
/// Returns None if no games need claiming.
async fn calculate_next_claim(
    auto_claim_games: &[(GameId, Option<ClaimTime>)],
    state: &AppState,
) -> Option<(Duration, Vec<GameId>)> {
    let now = Utc::now();
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
    let duration = (earliest - now).to_std().ok()?;

    Some((duration, games_at_earliest))
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // is_retryable_error — positive cases (each keyword)
    // =========================================================================

    #[test]
    fn retryable_timeout() {
        assert!(is_retryable_error("request timeout"));
    }

    #[test]
    fn retryable_connection() {
        assert!(is_retryable_error("connection refused by host"));
    }

    #[test]
    fn retryable_network() {
        assert!(is_retryable_error("network error"));
    }

    #[test]
    fn retryable_dns() {
        assert!(is_retryable_error("dns resolution failed"));
    }

    #[test]
    fn retryable_reset() {
        assert!(is_retryable_error("connection reset by peer"));
    }

    #[test]
    fn retryable_refused() {
        assert!(is_retryable_error("connection refused"));
    }

    #[test]
    fn retryable_unreachable() {
        assert!(is_retryable_error("host unreachable"));
    }

    // =========================================================================
    // is_retryable_error — case insensitivity
    // =========================================================================

    #[test]
    fn retryable_case_insensitive_uppercase() {
        assert!(is_retryable_error("CONNECTION TIMEOUT"));
    }

    #[test]
    fn retryable_case_insensitive_mixed() {
        assert!(is_retryable_error("Network Error: DNS lookup failed"));
    }

    // =========================================================================
    // is_retryable_error — negative cases
    // =========================================================================

    #[test]
    fn not_retryable_auth_error() {
        assert!(!is_retryable_error("authentication failed: invalid token"));
    }

    #[test]
    fn not_retryable_rate_limit() {
        assert!(!is_retryable_error("rate limit exceeded"));
    }

    #[test]
    fn not_retryable_invalid_cookie() {
        assert!(!is_retryable_error("invalid cookie"));
    }

    #[test]
    fn not_retryable_empty_string() {
        assert!(!is_retryable_error(""));
    }
}
