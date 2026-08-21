//! Honkai: Star Rail game implementation for Storekeeper.
//!
//! This crate provides the game client for fetching HSR resources
//! from the HoYoLab API.

mod client;
mod error;
mod resource;

pub use client::HsrClient;
pub use error::Error;
pub use error::Result;
pub use resource::HsrResource;
