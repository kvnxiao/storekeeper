//! Wuthering Waves game implementation for Storekeeper.
//!
//! This crate provides the game client for fetching WuWa resources
//! from the Kuro Games API.

mod client;
mod error;
mod resource;

pub use client::WuwaClient;
pub use error::Error;
pub use error::Result;
pub use resource::WuwaResource;
