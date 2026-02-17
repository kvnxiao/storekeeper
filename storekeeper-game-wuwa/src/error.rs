//! Error types for the Wuthering Waves game client.

/// Error type for WuWa operations.
///
/// Re-exports the Kuro client error directly since the game client
/// adds no game-specific error variants.
pub type Error = storekeeper_client_kuro::Error;

/// Result type alias using the WuWa Error type.
pub type Result<T> = std::result::Result<T, Error>;
