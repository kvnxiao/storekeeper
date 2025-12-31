//! Zenless Zone Zero game implementation for Storekeeper.
//!
//! This crate provides the game client for fetching ZZZ resources
//! from the HoYoLab API.

pub mod client;
pub mod error;
pub mod resource;

pub use client::ZzzClient;
pub use error::{Error, Result};
pub use resource::ZzzResource;
