//! System tray setup and event handling.

use tauri::{
    App, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

/// Sets up the system tray icon and menu.
///
/// # Errors
///
/// Returns an error if the tray icon or menu cannot be created.
pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    // Create menu items
    let refresh = MenuItem::with_id(app, "refresh", "Refresh Now", true, None::<&str>)?;
    let open_config =
        MenuItem::with_id(app, "open_config", "Open Config Folder", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Build the menu
    let menu = Menu::with_items(app, &[&refresh, &open_config, &quit])?;

    // Build the tray icon
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Storekeeper")
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "refresh" => {
                    // TODO: Trigger refresh
                    println!("Refresh requested");
                }
                "open_config" => {
                    // Open config folder
                    if let Err(e) = crate::commands::open_config_folder() {
                        eprintln!("Failed to open config folder: {e}");
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
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
        })
        .build(app)?;

    Ok(())
}
