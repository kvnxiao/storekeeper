//! Honkai: Star Rail game implementation for Storekeeper.
//!
//! This crate provides the game client for fetching HSR resources
//! from the HoYoLab API.

pub mod client;
pub mod error;
pub mod resource;

pub use client::HsrClient;
pub use error::{Error, Result};
pub use resource::HsrResource;
