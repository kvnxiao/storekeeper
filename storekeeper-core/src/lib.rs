//! Core traits, types, and configuration for Storekeeper.
//!
//! This crate provides the foundational types and abstractions used across
//! all game implementations and the main application.

mod config;
mod daily_reward;
mod error;
mod game;
mod game_id;
mod macros;
mod region;
mod resource;
mod resource_types;

pub mod serde_utils;

pub use config::AppConfig;
pub use config::ClaimTime;
pub use config::DEFAULT_AUTO_CLAIM_TIME;
pub use config::GamesConfig;
pub use config::GeneralConfig;
pub use config::GenshinConfig;
pub use config::HoyolabSecrets;
pub use config::HsrConfig;
pub use config::KuroSecrets;
pub use config::ResourceNotificationConfig;
pub use config::SecretsConfig;
pub use config::WuwaConfig;
pub use config::ZzzConfig;
pub use config::ensure_configs_exist;
pub use config::next_claim_datetime_utc;
pub use daily_reward::ClaimResult;
pub use daily_reward::DailyReward;
pub use daily_reward::DailyRewardClient;
pub use daily_reward::DailyRewardInfo;
pub use daily_reward::DailyRewardStatus;
pub use daily_reward::DynDailyRewardClient;
pub use error::Error;
pub use error::Result;
pub use game::DynGameClient;
pub use game::GameClient;
pub use game_id::ApiProvider;
pub use game_id::GameId;
pub use region::Region;
pub use resource::CooldownResource;
pub use resource::DisplayableResource;
pub use resource::ExpeditionResource;
pub use resource::StaminaResource;
pub use resource_types::GenshinResourceType;
pub use resource_types::HsrResourceType;
pub use resource_types::WuwaResourceType;
pub use resource_types::ZzzResourceType;
