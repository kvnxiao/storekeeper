//! Error types for the Honkai: Star Rail game client.

use thiserror::Error;

/// Error type for HSR operations.
#[derive(Error, Debug)]
pub enum Error {
    /// HoYoLab API error.
    #[error("HoYoLab API error: {0}")]
    HoyolabApi(#[from] storekeeper_client_hoyolab::Error),

    /// Core error.
    #[error("Core error: {0}")]
    Core(#[from] storekeeper_core::Error),

    /// Failed to parse API response.
    #[error("Failed to parse response: {0}")]
    ParseFailed(String),
}

/// Result type alias using the HSR Error type.
pub type Result<T> = std::result::Result<T, Error>;
