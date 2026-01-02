//! Genshin Impact game implementation for Storekeeper.
//!
//! This crate provides the game client for fetching Genshin Impact resources
//! from the HoYoLab API.

pub mod client;
pub mod error;
pub mod resource;

pub use client::GenshinClient;
pub use error::{Error, Result};
pub use resource::GenshinResource;
