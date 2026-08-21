//! Application windows beyond the one declared in `tauri.conf.json`.

use crate::i18n;
use anyhow::Context;
use anyhow::Result;
use tauri::AppHandle;
use tauri::Manager;
use tauri::WebviewUrl;
use tauri::WebviewWindowBuilder;

/// Label of the primary window, declared in `tauri.conf.json`.
pub const MAIN_WINDOW_LABEL: &str = "main";

pub const LOGS_WINDOW_LABEL: &str = "logs";

/// Opens the log viewer window, or reveals the one already open.
///
/// The window loads the `/logs` route, which a bundled build serves from the
/// prerendered `logs/index.html`.
///
/// # Errors
///
/// Returns an error if the window cannot be created or brought to the front.
pub fn open_logs_window(app_handle: &AppHandle) -> Result<()> {
    if let Some(window) = app_handle.get_webview_window(LOGS_WINDOW_LABEL) {
        window
            .unminimize()
            .context("failed to unminimize the log window")?;
        window.show().context("failed to show the log window")?;
        window
            .set_focus()
            .context("failed to focus the log window")?;
        return Ok(());
    }

    WebviewWindowBuilder::new(
        app_handle,
        LOGS_WINDOW_LABEL,
        WebviewUrl::App(LOGS_WINDOW_LABEL.into()),
    )
    .title(i18n::t("logs_window_title"))
    .inner_size(900.0, 600.0)
    .min_inner_size(480.0, 320.0)
    .build()
    .context("failed to create the log window")?;

    Ok(())
}
