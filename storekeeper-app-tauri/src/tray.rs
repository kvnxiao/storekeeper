//! System tray setup and event handling.

use anyhow::{Context, Result};
use tauri::{
    App, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
};
use tokio_util::sync::CancellationToken;

/// Sets up the system tray icon and menu.
///
/// # Errors
///
/// Returns an error if the tray icon or menu cannot be created.
pub fn setup_tray(app: &App) -> Result<()> {
    // Create menu items
    let refresh = MenuItem::with_id(app, "refresh", "Refresh Now", true, None::<&str>)
        .context("failed to create 'Refresh Now' menu item")?;
    let open_config =
        MenuItem::with_id(app, "open_config", "Open Config Folder", true, None::<&str>)
            .context("failed to create 'Open Config Folder' menu item")?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .context("failed to create 'Quit' menu item")?;

    // Build the menu
    let menu = Menu::with_items(app, &[&refresh, &open_config, &quit])
        .context("failed to create tray menu")?;

    // Get the tray icon defined in tauri.conf.json and configure it
    let tray = app
        .tray_by_id("main")
        .context("tray icon 'main' not found, ensure it's defined in tauri.conf.json")?;

    tray.set_menu(Some(menu))
        .context("failed to set tray menu")?;

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
