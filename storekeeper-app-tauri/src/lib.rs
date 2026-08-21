//! Storekeeper Tauri Application Library
//!
//! This module provides the main entry point and setup for the Tauri
//! application.

mod clients;
mod commands;
mod config_diff;
mod daily_reward_registry;
mod error;
mod events;
mod i18n;
mod logging;
mod notification;
mod polling;
mod provider_batch;
mod registry;
mod scheduled_claim;
mod state;
mod tray;
mod window;

use anyhow::Context;
use anyhow::Result;
use tauri::Manager;
use tauri::RunEvent;
use tauri_plugin_autostart::ManagerExt;
use tokio_util::sync::CancellationToken;
use window::MAIN_WINDOW_LABEL;

/// Applies the loaded configuration and starts the background tasks and the
/// tray icon.
///
/// # Errors
///
/// Returns an error if the tray icon cannot be created.
fn setup_app(
    app: &mut tauri::App,
    log_filter: logging::LogFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = state::AppState::with_config();

    let (language, log_level, should_autostart) = tauri::async_runtime::block_on(async {
        let inner = app_state.inner.read().await;
        (
            inner.config.general.language.clone(),
            inner.config.general.log_level.clone(),
            inner.config.general.autostart,
        )
    });

    log_filter.set_level(&log_level);
    app.manage(log_filter);

    let effective_locale = i18n::resolve_locale(language.as_deref());
    if let Err(e) = i18n::init(effective_locale) {
        tracing::warn!(error = %e, "Failed to initialize i18n, falling back to defaults");
        if let Err(e) = i18n::init("en") {
            tracing::error!(error = %e, "Failed to initialize i18n fallback locale");
        }
    }

    app.manage(app_state);

    let autolaunch = app.autolaunch();
    let autostart_result = if should_autostart {
        autolaunch.enable()
    } else {
        autolaunch.disable()
    };
    if let Err(e) = autostart_result {
        tracing::warn!(error = %e, "Failed to sync autostart state");
    }

    let cancel_token = CancellationToken::new();
    app.manage(cancel_token.clone());

    polling::start_polling(app.handle().clone(), cancel_token.clone());

    scheduled_claim::start_scheduled_claims(app.handle().clone(), cancel_token.clone());

    notification::start_notification_checker(app.handle().clone(), cancel_token.clone());

    setup_ctrlc_handler(app.handle().clone(), cancel_token);

    tray::setup_tray(app)?;

    Ok(())
}

/// Runs the Storekeeper application.
///
/// # Errors
///
/// Returns an error if the Tauri application fails to build.
#[expect(
    clippy::exit,
    reason = "tauri::generate_context! expands to a process::exit path"
)]
pub fn run() -> Result<()> {
    let log_filter = logging::init();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_dir = %logging::log_dir().unwrap_or_default(),
        "Storekeeper starting"
    );

    let app = tauri::Builder::default()
        // Must be registered first so it runs before any other plugin. When a
        // second instance is launched, this fires in the already-running
        // instance and the new process exits; reveal the existing window.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                if let Err(e) = window.unminimize() {
                    tracing::debug!(error = %e, "Failed to unminimize window");
                }
                if let Err(e) = window.show() {
                    tracing::debug!(error = %e, "Failed to show window");
                }
                if let Err(e) = window.set_focus() {
                    tracing::debug!(error = %e, "Failed to focus window");
                }
            }
        }))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| setup_app(app, log_filter))
        .invoke_handler(tauri::generate_handler![
            commands::get_all_resources,
            commands::refresh_resources,
            commands::get_config,
            commands::get_secrets,
            commands::save_and_apply,
            commands::open_config_folder,
            commands::open_log_folder,
            commands::open_logs_window,
            commands::read_log_tail,
            commands::send_preview_notification,
            commands::get_daily_reward_status,
            commands::refresh_daily_reward_status,
            commands::claim_daily_reward_for_game,
            commands::get_daily_reward_status_for_game,
            commands::get_supported_locales,
            commands::get_effective_locale,
        ])
        .on_window_event(|window, event| {
            // Only the main window survives its close button; a secondary
            // window kept alive hidden would keep polling the backend.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && window.label() == MAIN_WINDOW_LABEL
            {
                api.prevent_close();
                if let Err(e) = window.hide() {
                    tracing::debug!(error = %e, "Failed to hide window");
                }
            }
        })
        .build(tauri::generate_context!())
        .context("error while building tauri application")?;

    app.run(|app_handle, event| {
        if let RunEvent::ExitRequested { code, api, .. } = &event {
            tracing::info!(exit_code = ?code, "Application exit requested");

            if let Some(cancel_token) = app_handle.try_state::<CancellationToken>()
                && !cancel_token.is_cancelled()
            {
                tracing::info!("Cancelling background tasks...");
                cancel_token.cancel();
            }

            // The exit proceeds unless api.prevent_exit() is called.
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
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                tracing::info!("Ctrl+C received, initiating graceful shutdown...");

                cancel_token.cancel();

                // Let cancelled tasks finish cleanup before the process exits.
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                app_handle.exit(0);
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to listen for Ctrl+C signal");
            }
        }
    });
}
