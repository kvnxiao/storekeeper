//! Kuro Games API client for Storekeeper.
//!
//! This crate provides an HTTP client for interacting with the Kuro Games API,
//! used by Wuthering Waves.

mod cache;
mod client;
mod error;

pub use cache::load_oauth_from_cache;
pub use client::KuroClient;
pub use error::ClientError;
pub use error::Error;
pub use error::Result;
