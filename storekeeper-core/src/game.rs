//! Game client traits for fetching resources from game APIs.

use std::future::Future;
use std::pin::Pin;

use serde::Serialize;

use crate::game_id::GameId;

/// Type alias for a boxed error with Send + Sync bounds.
type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// A boxed future for object-safe async trait methods.
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait for game API clients.
///
/// Each game implementation provides a client that can fetch resource data
/// from the game's API.
pub trait GameClient: Send + Sync {
    /// The resource type returned by this game client.
    type Resource: Send + Serialize;

    /// The error type for this game client.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Returns the unique identifier for this game.
    fn game_id(&self) -> GameId;

    /// Returns the display name for this game.
    fn game_name(&self) -> &'static str {
        self.game_id().display_name()
    }

    /// Fetches all tracked resources from the game API.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the response cannot be parsed.
    #[must_use = "this performs an API call; the result should be used"]
    fn fetch_resources(
        &self,
    ) -> impl Future<Output = std::result::Result<Vec<Self::Resource>, Self::Error>> + Send;

    /// Checks if the client is properly authenticated.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication check fails.
    #[must_use = "this performs an API call; the result should be checked"]
    fn is_authenticated(
        &self,
    ) -> impl Future<Output = std::result::Result<bool, Self::Error>> + Send;
}

/// Type-erased game client for dynamic dispatch.
///
/// This trait allows storing different game clients in a single collection
/// by erasing the associated types. Resources are serialized to JSON values.
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
    fn fetch_resources_json(
        &self,
    ) -> BoxFuture<'_, std::result::Result<serde_json::Value, BoxError>>;

    /// Checks if the client is properly authenticated.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication check fails.
    fn is_authenticated_dyn(&self) -> BoxFuture<'_, std::result::Result<bool, BoxError>>;
}

/// Blanket implementation of `DynGameClient` for all `GameClient` implementors.
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

    fn fetch_resources_json(
        &self,
    ) -> BoxFuture<'_, std::result::Result<serde_json::Value, BoxError>> {
        Box::pin(async {
            let resources = self
                .fetch_resources()
                .await
                .map_err(|e| Box::new(e) as BoxError)?;
            serde_json::to_value(resources).map_err(|e| Box::new(e) as BoxError)
        })
    }

    fn is_authenticated_dyn(&self) -> BoxFuture<'_, std::result::Result<bool, BoxError>> {
        Box::pin(async {
            self.is_authenticated()
                .await
                .map_err(|e| Box::new(e) as BoxError)
        })
    }
}
