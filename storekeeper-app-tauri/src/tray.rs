//! System tray setup and event handling.

use anyhow::{Context, Result};
use tauri::{
    App, AppHandle, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
};
use tokio_util::sync::CancellationToken;

use crate::i18n;

/// Builds (or rebuilds) the tray menu with localized strings.
///
/// Can be called at startup and again after locale changes.
///
/// # Errors
///
/// Returns an error if the menu items or menu cannot be created,
/// or if the tray icon is not found.
pub fn build_tray_menu(app: &AppHandle) -> Result<()> {
    let refresh_label = i18n::t("tray.refresh_now");
    let open_config_label = i18n::t("tray.open_config_folder");
    let quit_label = i18n::t("tray.quit");

    let refresh = MenuItem::with_id(app, "refresh", &refresh_label, true, None::<&str>)
        .context("failed to create 'Refresh Now' menu item")?;
    let open_config = MenuItem::with_id(app, "open_config", &open_config_label, true, None::<&str>)
        .context("failed to create 'Open Config Folder' menu item")?;
    let quit = MenuItem::with_id(app, "quit", &quit_label, true, None::<&str>)
        .context("failed to create 'Quit' menu item")?;

    let menu = Menu::with_items(app, &[&refresh, &open_config, &quit])
        .context("failed to create tray menu")?;

    let tray = app
        .tray_by_id("main")
        .context("tray icon 'main' not found, ensure it's defined in tauri.conf.json")?;

    tray.set_menu(Some(menu))
        .context("failed to set tray menu")?;

    Ok(())
}

/// Sets up the system tray icon and menu.
///
/// # Errors
///
/// Returns an error if the tray icon or menu cannot be created.
pub fn setup_tray(app: &App) -> Result<()> {
    // Build initial menu using the app handle
    build_tray_menu(app.handle())?;

    // Get the tray icon and attach event handlers
    let tray = app
        .tray_by_id("main")
        .context("tray icon 'main' not found, ensure it's defined in tauri.conf.json")?;

    tray.on_menu_event(|app, event| {
        match event.id.as_ref() {
            "refresh" => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::polling::refresh_now(&app_handle).await {
                        tracing::error!("Refresh failed: {e}");
                    }
                });
            }
            "open_config" => {
                // Open config folder
                if let Err(e) = crate::commands::open_config_folder() {
                    tracing::error!("Failed to open config folder: {e}");
                }
            }
            "quit" => {
                tracing::info!("Quit requested from tray menu");

                // Cancel background tasks before exiting
                if let Some(cancel_token) = app.try_state::<CancellationToken>() {
                    cancel_token.cancel();
                }

                app.exit(0);
            }
            _ => {}
        }
    });

    tray.on_tray_icon_event(|tray, event| {
        match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                // Toggle window visibility on left click
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                // Show window on double click
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        }
    });

    Ok(())
}
