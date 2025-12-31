//! Error types for the Kuro Games API client.

use thiserror::Error;

// Re-export base error for convenience
pub use storekeeper_client_core::ClientError;

/// Error type for Kuro Games API operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Base client error (HTTP, deserialization, API errors).
    #[error(transparent)]
    Client(#[from] ClientError),

    /// Server requested retry (code 1005).
    #[error("Server requested retry (code 1005)")]
    RetryRequested,

    /// Failed to parse nested JSON data.
    #[error("Failed to parse nested data: {0}")]
    NestedDataParseFailed(String),
}

/// Result type alias using the Kuro Error type.
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

impl From<reqwest_middleware::Error> for Error {
    fn from(err: reqwest_middleware::Error) -> Self {
        match err {
            reqwest_middleware::Error::Reqwest(e) => Self::Client(ClientError::from(e)),
            reqwest_middleware::Error::Middleware(e) => {
                Self::Client(ClientError::api_error(0, e.to_string()))
            }
        }
    }
}
