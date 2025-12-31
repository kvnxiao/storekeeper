//! Error types for the HoYoLab API client.

use thiserror::Error;

// Re-export base error for convenience
pub use storekeeper_client_core::ClientError;

/// Error type for HoYoLab API operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Base client error (HTTP, deserialization, API errors).
    #[error(transparent)]
    Client(#[from] ClientError),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded, retry after {retry_after_secs} seconds")]
    RateLimited {
        /// Seconds to wait before retrying.
        retry_after_secs: u64,
    },
}

/// Result type alias using the HoYoLab Error type.
pub type Result<T> = std::result::Result<T, Error>;

// Convenience conversions for common base error types
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Client(ClientError::from(err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Client(ClientError::from(err))
    }
}
