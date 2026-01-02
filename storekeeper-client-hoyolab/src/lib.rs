//! HoYoLab API client for Storekeeper.
//!
//! This crate provides a shared HTTP client for interacting with the HoYoLab API,
//! used by Genshin Impact, Honkai: Star Rail, and Zenless Zone Zero.

pub mod client;
pub mod ds;
pub mod error;

pub use client::HoyolabClient;
pub use error::{Error, Result};

// Re-export reqwest::Method for use in game crates
pub use reqwest::Method;
