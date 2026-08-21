//! Zenless Zone Zero game implementation for Storekeeper.
//!
//! This crate provides the game client for fetching ZZZ resources
//! from the HoYoLab API.

mod client;
mod error;
mod resource;

pub use client::ZzzClient;
pub use error::Error;
pub use error::Result;
pub use resource::ZzzResource;
