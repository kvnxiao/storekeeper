//! Game client traits for fetching resources from game APIs.

use async_trait::async_trait;
use serde::Serialize;

use crate::game_id::GameId;

/// Trait for game API clients.
///
/// Each game implementation provides a client that can fetch resource data
/// from the game's API.
#[async_trait]
pub trait GameClient: Send + Sync {
    /// The resource type returned by this game client.
    type Resource: Send + Serialize;

    /// The error type for this game client.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Returns the unique identifier for this game.
    fn game_id(&self) -> GameId;

    /// Returns the display name for this game.
    fn game_name(&self) -> &'static str;

    /// Fetches all tracked resources from the game API.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response cannot be parsed.
    async fn fetch_resources(&self) -> std::result::Result<Vec<Self::Resource>, Self::Error>;

    /// Checks if the client is properly authenticated.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication check fails.
    async fn is_authenticated(&self) -> std::result::Result<bool, Self::Error>;
}

/// Type-erased game client for dynamic dispatch.
///
/// This trait allows storing different game clients in a single collection
/// by erasing the associated types. Resources are serialized to JSON values.
#[async_trait]
pub trait DynGameClient: Send + Sync {
    /// Returns the unique identifier for this game.
    fn game_id(&self) -> GameId;

    /// Returns the display name for this game.
    fn game_name(&self) -> &'static str;

    /// Fetches resources as a JSON value (type-erased).
    ///
    /// # Errors
    ///
    /// Returns an error if the fetch fails or serialization fails.
    async fn fetch_resources_json(
        &self,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;

    /// Checks if the client is properly authenticated.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication check fails.
    async fn is_authenticated_dyn(
        &self,
    ) -> std::result::Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}

/// Blanket implementation of `DynGameClient` for all `GameClient` implementors.
#[async_trait]
impl<T> DynGameClient for T
where
    T: GameClient,
{
    fn game_id(&self) -> GameId {
        GameClient::game_id(self)
    }

    fn game_name(&self) -> &'static str {
        GameClient::game_name(self)
    }

    async fn fetch_resources_json(
        &self,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let resources = self.fetch_resources().await.map_err(|e| {
            let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
            boxed
        })?;
        serde_json::to_value(resources).map_err(|e| {
            let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
            boxed
        })
    }

    async fn is_authenticated_dyn(
        &self,
    ) -> std::result::Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        self.is_authenticated().await.map_err(|e| {
            let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
            boxed
        })
    }
}
