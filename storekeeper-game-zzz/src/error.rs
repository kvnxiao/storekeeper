//! Error types for the Zenless Zone Zero game client.

/// Error type for ZZZ operations.
///
/// Re-exports the HoYoLab client error directly since the game client
/// adds no game-specific error variants.
pub type Error = storekeeper_client_hoyolab::Error;

/// Result type alias using the ZZZ Error type.
pub type Result<T> = std::result::Result<T, Error>;
