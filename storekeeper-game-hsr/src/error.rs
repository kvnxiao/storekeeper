//! Error types for the Honkai: Star Rail game client.

/// Error type for HSR operations.
///
/// Re-exports the HoYoLab client error directly since the game client
/// adds no game-specific error variants.
pub type Error = storekeeper_client_hoyolab::Error;

/// Result type alias using the HSR Error type.
pub type Result<T> = std::result::Result<T, Error>;
