//! HoYoLab API client for Storekeeper.
//!
//! This crate provides a shared HTTP client for interacting with the HoYoLab
//! API, used by Genshin Impact, Honkai: Star Rail, and Zenless Zone Zero.

mod client;
mod daily_reward;
mod ds;
mod error;
mod retcode;

pub use client::HoyolabClient;
pub use daily_reward::GENSHIN_DAILY_REWARD;
pub use daily_reward::HSR_DAILY_REWARD;
pub use daily_reward::HoyolabDailyRewardClient;
pub use daily_reward::HoyolabDailyRewardConfig;
pub use daily_reward::ZZZ_DAILY_REWARD;
pub use ds::generate_dynamic_secret_chinese;
pub use ds::generate_dynamic_secret_overseas;
pub use error::ClientError;
pub use error::Error;
pub use error::Result;
pub use reqwest::Method;
