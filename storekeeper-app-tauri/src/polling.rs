//! Background polling for periodic resource updates.

use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;

use crate::events::AppEvent;
use crate::state::{AllResources, AppState};

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
        let _ = poll_resources(&app_handle).await;

        loop {
            tokio::select! {
                () = cancel_token.cancelled() => {
                    tracing::info!("Background polling cancelled");
                    break;
                }
                () = tokio::time::sleep(poll_interval) => {
                    tracing::debug!("Scheduled poll triggered");
                    let _ = poll_resources(&app_handle).await;
                }
            }
        }
    });
}

/// Polls all resources and updates the state.
async fn poll_resources(app_handle: &AppHandle) -> Result<(), String> {
    let state = app_handle.state::<AppState>();

    // Check if already refreshing
    if state.is_refreshing().await {
        tracing::debug!("Skipping poll - refresh already in progress");
        return Ok(());
    }

    // Skip if no clients configured
    if !state.has_clients().await {
        tracing::debug!("Skipping poll - no game clients configured");
        return Ok(());
    }

    state.set_refreshing(true).await;

    tracing::debug!("Fetching resources from all game clients");

    // Fetch resources from all configured game clients
    let resources = state.fetch_all_resources(app_handle).await;

    state.set_resources(resources.clone()).await;
    state.set_refreshing(false).await;

    tracing::debug!("Resources updated, emitting event to frontend");

    // Emit event to frontend
    let _ = app_handle.emit(AppEvent::ResourcesUpdated.as_str(), &resources);

    Ok(())
}

/// Manually triggers a resource refresh.
///
/// This is called by the refresh command and tray menu action.
pub async fn refresh_now(app_handle: &AppHandle) -> Result<AllResources, String> {
    tracing::info!("Manual refresh requested");
    let state = app_handle.state::<AppState>();

    // Check if already refreshing
    if state.is_refreshing().await {
        tracing::debug!("Refresh already in progress, rejecting manual refresh");
        return Err("Refresh already in progress".to_string());
    }

    // If no clients configured, just return empty resources with timestamp
    if !state.has_clients().await {
        tracing::debug!("No game clients configured, returning empty resources");
        let mut resources = state.get_resources().await;
        resources.last_updated = Some(chrono::Utc::now());
        return Ok(resources);
    }

    // Emit refresh started event to frontend
    let _ = app_handle.emit(AppEvent::RefreshStarted.as_str(), ());

    state.set_refreshing(true).await;

    tracing::debug!("Fetching resources from all game clients");

    // Fetch resources from all configured game clients
    let resources = state.fetch_all_resources(app_handle).await;

    state.set_resources(resources.clone()).await;
    state.set_refreshing(false).await;

    tracing::info!("Manual refresh completed");

    // Emit event to frontend
    let _ = app_handle.emit(AppEvent::ResourcesUpdated.as_str(), &resources);

    Ok(resources)
}
