//! Core traits, types, and configuration for Storekeeper.
//!
//! This crate provides the foundational types and abstractions used across
//! all game implementations and the main application.

pub mod config;
pub mod daily_reward;
pub mod error;
pub mod game;
pub mod game_id;
pub mod macros;
pub mod region;
pub mod resource;
pub mod resource_types;
pub mod serde_utils;

pub use config::{
    AppConfig, ClaimTime, DEFAULT_AUTO_CLAIM_TIME, GamesConfig, GenshinConfig, HsrConfig,
    ResourceNotificationConfig, SecretsConfig, WuwaConfig, ZzzConfig, ensure_configs_exist,
    next_claim_datetime_utc,
};
pub use daily_reward::{
    ClaimResult, DailyReward, DailyRewardClient, DailyRewardInfo, DailyRewardStatus,
    DynDailyRewardClient,
};
pub use error::{Error, Result};
pub use game::{DynGameClient, GameClient};
pub use game_id::{ApiProvider, GameId};
pub use region::Region;
pub use resource::{CooldownResource, DisplayableResource, ExpeditionResource, StaminaResource};
pub use resource_types::{GenshinResourceType, HsrResourceType, WuwaResourceType, ZzzResourceType};
