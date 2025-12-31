//! Core traits, types, and configuration for Storekeeper.
//!
//! This crate provides the foundational types and abstractions used across
//! all game implementations and the main application.

pub mod config;
pub mod error;
pub mod game;
pub mod game_id;
pub mod macros;
pub mod region;
pub mod resource;

pub use config::{AppConfig, NotificationConfig, SecretsConfig, ensure_configs_exist};
pub use error::{Error, Result};
pub use game::{DynGameClient, GameClient};
pub use game_id::{ApiProvider, GameId};
pub use region::Region;
pub use resource::{CooldownResource, DisplayableResource, ExpeditionResource, StaminaResource};
