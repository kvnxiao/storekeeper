//! Error types for the Wuthering Waves game client.

use thiserror::Error;

/// Error type for WuWa operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Kuro API error.
    #[error("Kuro API error: {0}")]
    KuroApi(#[from] storekeeper_client_kuro::Error),
}

/// Result type alias using the WuWa Error type.
pub type Result<T> = std::result::Result<T, Error>;
