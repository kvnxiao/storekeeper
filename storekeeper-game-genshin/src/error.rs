//! Error types for the Genshin Impact game client.

use thiserror::Error;

/// Error type for Genshin Impact operations.
#[derive(Error, Debug)]
pub enum Error {
    /// HoYoLab API error.
    #[error("HoYoLab API error: {0}")]
    HoyolabApi(#[from] storekeeper_client_hoyolab::Error),
}

/// Result type alias using the Genshin Error type.
pub type Result<T> = std::result::Result<T, Error>;
