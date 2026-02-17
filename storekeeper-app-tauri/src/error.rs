//! Structured error types for Tauri commands.
//!
//! Provides typed error codes that the frontend can switch on, replacing
//! opaque `String` errors.

use serde::Serialize;

/// Error codes for frontend consumption.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// Configuration file not found.
    ConfigNotFound,
    /// Configuration file is malformed or invalid.
    ConfigInvalid,
    /// Filesystem I/O error.
    IoError,
    /// OS notification system error.
    NotificationError,
    /// Unclassified internal error.
    Internal,
}

/// Structured error for Tauri commands.
///
/// Serializes as `{ "code": "CONFIG_NOT_FOUND", "message": "..." }` so the
/// frontend can differentiate error types.
#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    /// The error code identifying the type of error.
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
}

impl CommandError {
    /// Creates a new `CommandError` with the `Internal` error code.
    #[must_use = "this returns a new CommandError instance"]
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Internal,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<storekeeper_core::Error> for CommandError {
    fn from(err: storekeeper_core::Error) -> Self {
        let code = match &err {
            storekeeper_core::Error::ConfigNotFound { .. } => ErrorCode::ConfigNotFound,
            storekeeper_core::Error::ConfigParseFailed { .. }
            | storekeeper_core::Error::TomlDeserialize(_)
            | storekeeper_core::Error::ValidationError { .. }
            | storekeeper_core::Error::InvalidRegion(_)
            | storekeeper_core::Error::UnknownUidRegion(_) => ErrorCode::ConfigInvalid,
            storekeeper_core::Error::ConfigReadFailed(_) => ErrorCode::IoError,
        };
        Self {
            code,
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        Self {
            code: ErrorCode::IoError,
            message: err.to_string(),
        }
    }
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        Self::internal(message)
    }
}

impl From<anyhow::Error> for CommandError {
    fn from(err: anyhow::Error) -> Self {
        Self::internal(format!("{err:#}"))
    }
}
