//! Background polling for periodic resource updates.

use std::collections::HashSet;
use std::time::Duration;

use chrono::Utc;
use storekeeper_core::GameId;
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;

use crate::events::AppEvent;
use crate::notification;
use crate::state::{AllResources, AppState};

/// RAII guard that resets the refresh-in-progress flag on drop.
struct RefreshGuard<'a> {
    state: &'a AppState,
}

impl Drop for RefreshGuard<'_> {
    fn drop(&mut self) {
        self.state.finish_refresh();
    }
}

/// Tries to acquire the refresh slot, returning a guard on success.
fn try_acquire_refresh(state: &AppState) -> Option<RefreshGuard<'_>> {
    if state.try_start_refresh() {
        Some(RefreshGuard { state })
    } else {
        None
    }
}

/// Starts the background polling task.
///
/// This spawns a tokio task that periodically fetches resources from all
/// configured game APIs and emits update events to the frontend.
pub fn start_polling(app_handle: AppHandle, cancel_token: CancellationToken) {
    tauri::async_runtime::spawn(async move {
        // Get poll interval from config
        let state = app_handle.state::<AppState>();
        let poll_interval_secs = state.poll_interval_secs().await;
        let poll_interval = Duration::from_secs(poll_interval_secs);

        tracing::info!(
            poll_interval_secs = poll_interval_secs,
            "Starting background polling task"
        );

        // Initial fetch after short delay
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Do an initial fetch on startup
        tracing::debug!("Performing initial resource fetch");
        try_refresh(&app_handle).await;

        loop {
            tokio::select! {
                () = cancel_token.cancelled() => {
                    tracing::info!("Background polling cancelled");
                    break;
                }
                () = tokio::time::sleep(poll_interval) => {
                    tracing::debug!("Scheduled poll triggered");
                    try_refresh(&app_handle).await;
                }
            }
        }
    });
}

/// Attempts a refresh, skipping if already refreshing or no clients configured.
async fn try_refresh(app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();

    let Some(_refresh_guard) = try_acquire_refresh(&state) else {
        tracing::debug!("Skipping poll - refresh already in progress");
        return;
    };

    if !state.has_clients().await {
        tracing::debug!("Skipping poll - no game clients configured");
        return;
    }

    do_refresh(app_handle).await;
}

/// Core refresh logic shared by polling and manual refresh.
///
/// Fetches resources from all game clients, updates state, emits events,
/// and checks notification thresholds. Returns the fetched resources.
async fn do_refresh(app_handle: &AppHandle) -> AllResources {
    let state = app_handle.state::<AppState>();

    tracing::debug!("Fetching resources from all game clients");

    let resources = state.fetch_all_resources(app_handle).await;

    state.set_resources(resources.clone()).await;

    let _ = app_handle.emit(AppEvent::ResourcesUpdated.as_str(), &resources);

    notification::check_and_notify(app_handle).await;

    resources
}

/// Manually triggers a resource refresh.
///
/// This is called by the refresh command and tray menu action.
pub async fn refresh_now(app_handle: &AppHandle) -> Result<AllResources, String> {
    tracing::info!("Manual refresh requested");
    let state = app_handle.state::<AppState>();

    // Check if already refreshing
    let Some(_refresh_guard) = try_acquire_refresh(&state) else {
        tracing::debug!("Refresh already in progress, rejecting manual refresh");
        return Err("Refresh already in progress".to_string());
    };

    // If no clients configured, just return empty resources with timestamp
    if !state.has_clients().await {
        tracing::debug!("No game clients configured, returning empty resources");
        let mut resources = state.get_resources().await;
        resources.last_updated = Some(chrono::Utc::now());
        return Ok(resources);
    }

    // Emit refresh started event to frontend
    let _ = app_handle.emit(AppEvent::RefreshStarted.as_str(), ());

    let resources = do_refresh(app_handle).await;

    tracing::info!("Manual refresh completed");

    Ok(resources)
}

/// Refreshes resources and daily reward status for a specific set of games.
///
/// Merges the fetched results into the existing cached state rather than
/// replacing it entirely. This is used by the config reload path to only
/// fetch games whose configuration actually changed.
pub async fn refresh_games(
    app_handle: &AppHandle,
    game_ids: &HashSet<GameId>,
) -> Result<AllResources, String> {
    tracing::info!(games = ?game_ids, "Selective refresh requested");
    let state = app_handle.state::<AppState>();

    let Some(_refresh_guard) = try_acquire_refresh(&state) else {
        tracing::debug!("Refresh already in progress, skipping selective refresh");
        return Err("Refresh already in progress".to_string());
    };

    let _ = app_handle.emit(AppEvent::RefreshStarted.as_str(), ());

    // Fetch only the specified games
    let new_resources = state.fetch_resources_for_games(game_ids, app_handle).await;
    let new_daily_status = state.fetch_daily_reward_status_for_games(game_ids).await;

    // Merge into existing cached state
    let mut resources = state.get_resources().await;
    for (game_id, data) in new_resources {
        resources.games.insert(game_id, data);
    }
    resources.last_updated = Some(Utc::now());
    state.set_resources(resources.clone()).await;

    let mut daily_status = state.get_daily_reward_status().await;
    for (game_id, data) in new_daily_status {
        daily_status.games.insert(game_id, data);
    }
    daily_status.last_checked = Some(Utc::now());
    state.set_daily_reward_status(daily_status).await;

    // Emit full snapshot and run notification check
    let _ = app_handle.emit(AppEvent::ResourcesUpdated.as_str(), &resources);
    notification::check_and_notify(app_handle).await;

    tracing::info!("Selective refresh completed");
    Ok(resources)
}
