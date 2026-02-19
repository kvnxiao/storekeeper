//! Storekeeper Tauri Application Library
//!
//! This module provides the main entry point and setup for the Tauri application.

mod clients;
mod commands;
mod config_diff;
mod daily_reward_registry;
mod error;
mod events;
pub mod i18n;
mod notification;
mod polling;
mod provider_batch;
mod registry;
mod scheduled_claim;
mod state;
mod tray;

use anyhow::{Context, Result};
use tauri::{Manager, RunEvent};
use tauri_plugin_autostart::ManagerExt;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;

/// Initializes the tracing subscriber for logging.
///
/// Uses `RUST_LOG` environment variable if set, otherwise defaults to "info".
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

/// Runs the Storekeeper application.
///
/// # Errors
///
/// Returns an error if the Tauri application fails to build.
pub fn run() -> Result<()> {
    init_tracing();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Initialize application state with config and game clients
            let app_state = state::AppState::with_config();

            // Read config values needed for setup
            let (language, should_autostart) = tauri::async_runtime::block_on(async {
                let inner = app_state.inner.read().await;
                (
                    inner.config.general.language.clone(),
                    inner.config.general.autostart,
                )
            });

            // Initialize i18n with resolved locale (auto-detect if no override)
            let effective_locale = i18n::resolve_locale(language.as_deref());
            if let Err(e) = i18n::init(effective_locale) {
                tracing::warn!(error = %e, "Failed to initialize i18n, falling back to defaults");
                let _ = i18n::init("en");
            }

            app.manage(app_state);

            // Sync autostart state from config
            let autolaunch = app.autolaunch();
            let autostart_result = if should_autostart {
                autolaunch.enable()
            } else {
                autolaunch.disable()
            };
            if let Err(e) = autostart_result {
                tracing::warn!(error = %e, "Failed to sync autostart state");
            }

            // Create cancellation token for background tasks
            let cancel_token = CancellationToken::new();
            app.manage(cancel_token.clone());

            // Start background polling for resources
            polling::start_polling(app.handle().clone(), cancel_token.clone());

            // Start scheduled daily reward claims
            scheduled_claim::start_scheduled_claims(app.handle().clone(), cancel_token.clone());

            // Start notification checker
            notification::start_notification_checker(app.handle().clone(), cancel_token.clone());

            // Set up Ctrl+C handler to trigger graceful shutdown
            setup_ctrlc_handler(app.handle().clone(), cancel_token);

            // Set up system tray
            tray::setup_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_all_resources,
            commands::refresh_resources,
            commands::get_config,
            commands::get_secrets,
            commands::save_and_apply,
            commands::open_config_folder,
            // Notification commands
            commands::send_preview_notification,
            // Daily reward commands
            commands::get_daily_reward_status,
            commands::refresh_daily_reward_status,
            commands::claim_daily_rewards,
            commands::claim_daily_reward_for_game,
            commands::get_daily_reward_status_for_game,
            // Locale commands
            commands::get_supported_locales,
            commands::get_effective_locale,
        ])
        .on_window_event(|window, event| {
            // Handle close button - minimize to tray instead of closing
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Prevent the window from closing
                api.prevent_close();
                // Hide the window instead
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .context("error while building tauri application")?;

    // Run with custom event loop to handle graceful shutdown
    app.run(|app_handle, event| {
        if let RunEvent::ExitRequested { code, api, .. } = &event {
            tracing::info!(exit_code = ?code, "Application exit requested");

            // Cancel all background tasks
            if let Some(cancel_token) = app_handle.try_state::<CancellationToken>() {
                if !cancel_token.is_cancelled() {
                    tracing::info!("Cancelling background tasks...");
                    cancel_token.cancel();
                }
            }

            // Allow the exit to proceed (don't call api.prevent_exit())
            let _ = api;
        }
    });

    Ok(())
}

/// Sets up a Ctrl+C (SIGINT) handler to trigger graceful shutdown.
///
/// On Windows, this also handles console close events.
/// On Unix, this handles SIGINT and SIGTERM.
fn setup_ctrlc_handler(app_handle: tauri::AppHandle, cancel_token: CancellationToken) {
    tauri::async_runtime::spawn(async move {
        // Wait for Ctrl+C signal
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                tracing::info!("Ctrl+C received, initiating graceful shutdown...");

                // Cancel all background tasks
                cancel_token.cancel();

                // Give tasks a moment to clean up
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                // Exit the application
                app_handle.exit(0);
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to listen for Ctrl+C signal");
            }
        }
    });
}
