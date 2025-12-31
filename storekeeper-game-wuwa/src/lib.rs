//! Wuthering Waves game implementation for Storekeeper.
//!
//! This crate provides the game client for fetching WuWa resources
//! from the Kuro Games API.

pub mod client;
pub mod error;
pub mod resource;

pub use client::WuwaClient;
pub use error::{Error, Result};
pub use resource::WuwaResource;
