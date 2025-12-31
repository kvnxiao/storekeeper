//! Storekeeper Tauri Application Library
//!
//! This module provides the main entry point and setup for the Tauri application.

mod clients;
mod commands;
mod polling;
mod registry;
mod state;
mod tray;

use tauri::Manager;
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
/// # Panics
///
/// Panics if the Tauri application fails to build or run.
#[allow(clippy::missing_panics_doc, clippy::expect_used)]
pub fn run() {
    init_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Initialize application state with config and game clients
            let app_state = state::AppState::with_config();
            app.manage(app_state);

            // Create cancellation token for background tasks
            let cancel_token = CancellationToken::new();
            app.manage(cancel_token.clone());

            // Start background polling
            polling::start_polling(app.handle().clone(), cancel_token);

            // Set up system tray
            tray::setup_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_all_resources,
            commands::refresh_resources,
            commands::get_config,
            commands::open_config_folder,
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
