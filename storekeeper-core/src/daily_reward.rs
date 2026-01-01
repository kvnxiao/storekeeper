//! Daily reward types and traits for HoYoverse games.
//!
//! This module provides the core abstractions for daily check-in rewards
//! across Genshin Impact, Honkai: Star Rail, and Zenless Zone Zero.

use async_trait::async_trait;
use chrono::Datelike;
use serde::{Deserialize, Serialize};

use crate::game_id::GameId;

/// Information about the current daily reward status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRewardInfo {
    /// Whether today's reward has already been claimed.
    pub is_signed: bool,
    /// Total number of days signed in this month.
    pub total_sign_day: u32,
}

impl DailyRewardInfo {
    /// Creates a new `DailyRewardInfo`.
    #[must_use = "this returns a new DailyRewardInfo instance"]
    pub const fn new(is_signed: bool, total_sign_day: u32) -> Self {
        Self {
            is_signed,
            total_sign_day,
        }
    }

    /// Returns the number of missed rewards this month.
    ///
    /// Calculates based on the current day of month in UTC+8 timezone.
    #[must_use = "this returns the count of missed rewards"]
    pub fn missed_rewards(&self) -> u32 {
        use chrono::{FixedOffset, Utc};

        // Daily rewards reset at UTC+8 (China Standard Time)
        // SAFETY: 8 * 3600 = 28800 seconds is always a valid offset
        let Some(utc8) = FixedOffset::east_opt(8 * 3600) else {
            return 0; // Fallback if offset creation fails (should never happen)
        };
        let now = Utc::now().with_timezone(&utc8);
        let current_day = now.day();

        current_day.saturating_sub(self.total_sign_day)
    }
}

/// A claimable daily reward item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReward {
    /// Name of the reward item.
    pub name: String,
    /// Quantity of the reward.
    pub amount: u32,
    /// URL to the reward icon.
    pub icon: String,
}

impl DailyReward {
    /// Creates a new `DailyReward`.
    #[must_use = "this returns a new DailyReward instance"]
    pub fn new(name: impl Into<String>, amount: u32, icon: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            amount,
            icon: icon.into(),
        }
    }
}

/// Result of attempting to claim a daily reward.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimResult {
    /// Whether the claim was successful.
    pub success: bool,
    /// The reward that was claimed (if successful) or would have been claimed.
    pub reward: Option<DailyReward>,
    /// Updated sign-in status after the claim attempt.
    pub info: DailyRewardInfo,
    /// Optional message (e.g., "Already claimed today" or error details).
    pub message: Option<String>,
}

impl ClaimResult {
    /// Creates a successful claim result.
    #[must_use = "this returns a new ClaimResult instance"]
    pub fn success(reward: DailyReward, info: DailyRewardInfo) -> Self {
        Self {
            success: true,
            reward: Some(reward),
            info,
            message: None,
        }
    }

    /// Creates a failed claim result with a message.
    #[must_use = "this returns a new ClaimResult instance"]
    pub fn already_claimed(reward: Option<DailyReward>, info: DailyRewardInfo) -> Self {
        Self {
            success: false,
            reward,
            info,
            message: Some("Already claimed today".to_string()),
        }
    }

    /// Creates an error claim result.
    #[must_use = "this returns a new ClaimResult instance"]
    pub fn error(message: impl Into<String>, info: DailyRewardInfo) -> Self {
        Self {
            success: false,
            reward: None,
            info,
            message: Some(message.into()),
        }
    }
}

/// Full daily reward status including monthly rewards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRewardStatus {
    /// Current sign-in status.
    pub info: DailyRewardInfo,
    /// Today's reward (claimed or claimable).
    pub today_reward: Option<DailyReward>,
    /// All rewards available this month.
    pub monthly_rewards: Vec<DailyReward>,
}

impl DailyRewardStatus {
    /// Creates a new `DailyRewardStatus`.
    #[must_use = "this returns a new DailyRewardStatus instance"]
    pub fn new(
        info: DailyRewardInfo,
        today_reward: Option<DailyReward>,
        monthly_rewards: Vec<DailyReward>,
    ) -> Self {
        Self {
            info,
            today_reward,
            monthly_rewards,
        }
    }
}

/// Trait for game clients that support daily reward claiming.
///
/// This trait is separate from `GameClient` to allow games that may not
/// support daily rewards and to keep concerns separated.
#[async_trait]
pub trait DailyRewardClient: Send + Sync {
    /// The error type for this client.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Returns the game identifier for this client.
    fn game_id(&self) -> GameId;

    /// Gets the current daily reward status (signed today, total sign days).
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    async fn get_reward_info(&self) -> std::result::Result<DailyRewardInfo, Self::Error>;

    /// Gets the list of all monthly rewards.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    async fn get_monthly_rewards(&self) -> std::result::Result<Vec<DailyReward>, Self::Error>;

    /// Gets the full daily reward status including today's reward.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    async fn get_reward_status(&self) -> std::result::Result<DailyRewardStatus, Self::Error>;

    /// Claims the daily reward.
    ///
    /// Returns a `ClaimResult` indicating success or failure, along with
    /// the claimed reward details and updated status.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails (network error, auth error, etc.).
    /// Note: "Already claimed" is not an error - it's returned as a failed `ClaimResult`.
    async fn claim_daily_reward(&self) -> std::result::Result<ClaimResult, Self::Error>;
}

/// Type-erased trait for dynamic dispatch of daily reward clients.
///
/// This allows storing different game clients in a single collection.
#[async_trait]
pub trait DynDailyRewardClient: Send + Sync {
    /// Returns the game identifier for this client.
    fn game_id(&self) -> GameId;

    /// Gets the reward status as a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if the fetch or serialization fails.
    async fn get_reward_status_json(
        &self,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;

    /// Claims the daily reward and returns the result as JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if the claim or serialization fails.
    async fn claim_daily_reward_json(
        &self,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;
}

/// Blanket implementation of `DynDailyRewardClient` for all `DailyRewardClient` implementors.
#[async_trait]
impl<T> DynDailyRewardClient for T
where
    T: DailyRewardClient,
{
    fn game_id(&self) -> GameId {
        DailyRewardClient::game_id(self)
    }

    async fn get_reward_status_json(
        &self,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let status = self.get_reward_status().await.map_err(|e| {
            let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
            boxed
        })?;
        serde_json::to_value(status).map_err(|e| {
            let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
            boxed
        })
    }

    async fn claim_daily_reward_json(
        &self,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let result = self.claim_daily_reward().await.map_err(|e| {
            let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
            boxed
        })?;
        serde_json::to_value(result).map_err(|e| {
            let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
            boxed
        })
    }
}
