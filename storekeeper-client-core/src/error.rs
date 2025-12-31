//! Base error types for API clients.

use thiserror::Error;

/// Base error type shared by all API clients.
///
/// Client implementations should create their own error enum that wraps
/// this type via `#[from]` and adds client-specific variants.
#[derive(Error, Debug)]
pub enum ClientError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    /// Failed to deserialize API response.
    #[error("Failed to deserialize response: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// API returned an error response.
    #[error("API error (code {code}): {message}")]
    ApiError {
        /// Error code from the API.
        code: i32,
        /// Error message from the API.
        message: String,
    },

    /// Authentication failed or expired.
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

impl ClientError {
    /// Creates a new API error.
    #[must_use = "this returns a new ClientError and does not modify self"]
    pub fn api_error(code: i32, message: impl Into<String>) -> Self {
        Self::ApiError {
            code,
            message: message.into(),
        }
    }

    /// Creates a new authentication error.
    #[must_use = "this returns a new ClientError and does not modify self"]
    pub fn auth_failed(reason: impl Into<String>) -> Self {
        Self::AuthenticationFailed(reason.into())
    }

    /// Creates a new configuration error.
    #[must_use = "this returns a new ClientError and does not modify self"]
    pub fn invalid_config(reason: impl Into<String>) -> Self {
        Self::InvalidConfig(reason.into())
    }

    /// Returns the error code if this is an API error.
    #[must_use]
    pub fn api_error_code(&self) -> Option<i32> {
        match self {
            Self::ApiError { code, .. } => Some(*code),
            _ => None,
        }
    }
}

/// Result type alias using the base `ClientError` type.
pub type Result<T> = std::result::Result<T, ClientError>;
