//! Error types for the HoYoLab API client.

pub use storekeeper_client_core::ClientError;
use thiserror::Error;

/// Error type for HoYoLab API operations.
#[derive(Error, Debug)]
#[non_exhaustive]
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

    /// The sign endpoint accepted the request but the follow-up status remains
    /// unsigned.
    #[error("Sign accepted but the reward is still unclaimed (risk_code {risk_code:?})")]
    ClaimNotRegistered {
        /// Risk-control code returned by the sign endpoint, when present.
        risk_code: Option<i32>,
    },
}

/// Result type alias using the HoYoLab Error type.
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
