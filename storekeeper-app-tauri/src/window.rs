//! Application windows beyond the one declared in `tauri.conf.json`.

use crate::i18n;
use anyhow::Context;
use anyhow::Result;
use tauri::AppHandle;
use tauri::Manager;
use tauri::Theme;
use tauri::WebviewUrl;
use tauri::WebviewWindowBuilder;
use tauri::utils::config::Color;
use tauri::utils::config::WindowConfig;

/// Label of the primary window, declared in `tauri.conf.json`.
pub const MAIN_WINDOW_LABEL: &str = "main";

pub const LOGS_WINDOW_LABEL: &str = "logs";

const LOGS_WINDOW_WIDTH: f64 = 900.0;
const LOGS_WINDOW_HEIGHT: f64 = 600.0;
const LOGS_WINDOW_MIN_WIDTH: f64 = 480.0;
const LOGS_WINDOW_MIN_HEIGHT: f64 = 320.0;

/// `--background` from the stylesheet, painted before the webview's first
/// frame so the window does not open on a white flash.
const LOGS_BACKGROUND_LIGHT: Color = Color(255, 255, 255, 255);

const LOGS_BACKGROUND_DARK: Color = Color(9, 9, 11, 255);

fn logs_window_background(theme: Option<Theme>) -> Color {
    match theme {
        Some(Theme::Dark) => LOGS_BACKGROUND_DARK,
        _ => LOGS_BACKGROUND_LIGHT,
    }
}

/// Derives the log window's configuration from the main window's.
///
/// `WebView2` fails with `ERROR_INVALID_STATE` when a second webview requests
/// environment options differing from the environment already open over the
/// same data directory, and `scrollBarStyle` is one of those options, so the
/// log window inherits every field it does not override.
fn logs_window_config(main: &WindowConfig, theme: Option<Theme>) -> WindowConfig {
    WindowConfig {
        label: LOGS_WINDOW_LABEL.to_owned(),
        title: i18n::t("logs_window_title"),
        url: WebviewUrl::App(LOGS_WINDOW_LABEL.into()),
        width: LOGS_WINDOW_WIDTH,
        height: LOGS_WINDOW_HEIGHT,
        min_width: Some(LOGS_WINDOW_MIN_WIDTH),
        min_height: Some(LOGS_WINDOW_MIN_HEIGHT),
        center: true,
        visible: true,
        background_color: Some(logs_window_background(theme)),
        ..main.clone()
    }
}

/// Opens the log viewer window, or reveals the one already open.
///
/// The window loads the `/logs` route, which a bundled build serves from the
/// prerendered `logs/index.html`.
///
/// # Errors
///
/// Returns an error if `tauri.conf.json` declares no main window, and if the
/// window cannot be created or brought to the front.
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

    let theme = app_handle
        .get_webview_window(MAIN_WINDOW_LABEL)
        .and_then(|window| window.theme().ok());

    let config = app_handle
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == MAIN_WINDOW_LABEL)
        .map(|main| logs_window_config(main, theme))
        .context("tauri.conf.json declares no main window")?;

    WebviewWindowBuilder::from_config(app_handle, &config)
        .context("failed to configure the log window")?
        .build()
        .context("failed to create the log window")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::utils::config::ScrollBarStyle;

    fn main_window() -> WindowConfig {
        WindowConfig {
            scroll_bar_style: ScrollBarStyle::FluentOverlay,
            visible: false,
            ..WindowConfig::default()
        }
    }

    #[test]
    fn logs_window_config_inherits_the_main_window_webview_options() {
        let logs = logs_window_config(&main_window(), None);

        assert_eq!(logs.scroll_bar_style, ScrollBarStyle::FluentOverlay);
    }

    #[test]
    fn logs_window_config_overrides_the_identity_of_the_main_window() {
        let logs = logs_window_config(&main_window(), None);

        assert_eq!(logs.label, LOGS_WINDOW_LABEL);
        assert_eq!(logs.url, WebviewUrl::App(LOGS_WINDOW_LABEL.into()));
        assert!(logs.visible, "a hidden log window would never be shown");
    }

    #[test]
    fn logs_window_background_follows_the_reported_theme() {
        assert_eq!(
            logs_window_background(Some(Theme::Dark)),
            LOGS_BACKGROUND_DARK
        );
        assert_eq!(
            logs_window_background(Some(Theme::Light)),
            LOGS_BACKGROUND_LIGHT
        );
        assert_eq!(logs_window_background(None), LOGS_BACKGROUND_LIGHT);
    }
}
