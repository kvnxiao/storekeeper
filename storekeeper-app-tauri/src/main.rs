//! Storekeeper - Gacha Game Stamina Tracker
//!
//! A cross-platform desktop tray application for tracking gacha game stamina resources.

// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use anyhow::Result;

fn main() -> Result<()> {
    storekeeper_app_tauri::run()
}
