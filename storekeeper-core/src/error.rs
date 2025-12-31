//! Error types for the storekeeper-core crate.

use thiserror::Error;

/// Core error type for Storekeeper operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration file not found.
    #[error("Configuration file not found: {path}")]
    ConfigNotFound {
        /// Path to the missing configuration file.
        path: String,
    },

    /// Failed to parse configuration file.
    #[error("Failed to parse configuration: {0}")]
    ConfigParseFailed(String),

    /// Failed to read configuration file.
    #[error("Failed to read configuration file: {0}")]
    ConfigReadFailed(#[from] std::io::Error),

    /// TOML deserialization error.
    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    /// Validation error for configuration or input.
    #[error("Validation error: {field} {constraint}")]
    ValidationError {
        /// The field that failed validation.
        field: String,
        /// The constraint that was violated.
        constraint: String,
    },

    /// Invalid region specified.
    #[error("Invalid region: {0}")]
    InvalidRegion(String),

    /// Failed to determine region from UID.
    #[error("Could not determine region from UID: {0}")]
    UnknownUidRegion(String),
}

/// Result type alias using the core Error type.
pub type Result<T> = std::result::Result<T, Error>;
