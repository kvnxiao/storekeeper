//! Genshin Impact game implementation for Storekeeper.
//!
//! This crate provides the game client for fetching Genshin Impact resources
//! from the HoYoLab API.

mod client;
mod error;
mod resource;

pub use client::GenshinClient;
pub use error::Error;
pub use error::Result;
pub use resource::GenshinResource;
