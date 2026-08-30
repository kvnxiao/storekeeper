//! Error types for the Kuro Games API client.

pub use storekeeper_client_core::ClientError;
use thiserror::Error;

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

    /// An XOR-5 payload contains a byte outside ASCII.
    #[error("XOR-5 payload is not ASCII")]
    NonAsciiXor5Payload,

    /// Error from the storekeeper-core crate.
    #[error(transparent)]
    Core(#[from] storekeeper_core::Error),
}

/// Result type alias using the Kuro Error type.
pub type Result<T> = std::result::Result<T, Error>;
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
        Self::Client(ClientError::from(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_crosses_task_boundaries() {
        const fn assert_send<T: Send>() {}
        const fn assert_sync<T: Sync>() {}
        assert_send::<Error>();
        assert_sync::<Error>();
    }
}
