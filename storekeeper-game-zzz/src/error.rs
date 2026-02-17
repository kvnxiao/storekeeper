//! Error types for the Zenless Zone Zero game client.

use thiserror::Error;

/// Error type for ZZZ operations.
#[derive(Error, Debug)]
pub enum Error {
    /// HoYoLab API error.
    #[error("HoYoLab API error: {0}")]
    HoyolabApi(#[from] storekeeper_client_hoyolab::Error),
}

/// Result type alias using the ZZZ Error type.
pub type Result<T> = std::result::Result<T, Error>;
