//! Daily reward types and traits for HoYoverse games.
//!
//! This module provides the core abstractions for daily check-in rewards
//! across Genshin Impact, Honkai: Star Rail, and Zenless Zone Zero.

use chrono::Datelike;
use serde::{Deserialize, Serialize};

use crate::game_id::GameId;

/// Type alias for a boxed error with Send + Sync bounds.
type BoxError = Box<dyn std::error::Error + Send + Sync>;

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
    fn get_reward_info(
        &self,
    ) -> impl Future<Output = std::result::Result<DailyRewardInfo, Self::Error>> + Send;

    /// Gets the list of all monthly rewards.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    fn get_monthly_rewards(
        &self,
    ) -> impl Future<Output = std::result::Result<Vec<DailyReward>, Self::Error>> + Send;

    /// Gets the full daily reward status including today's reward.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    fn get_reward_status(
        &self,
    ) -> impl Future<Output = std::result::Result<DailyRewardStatus, Self::Error>> + Send;

    /// Claims the daily reward.
    ///
    /// Returns a `ClaimResult` indicating success or failure, along with
    /// the claimed reward details and updated status.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails (network error, auth error, etc.).
    /// Note: "Already claimed" is not an error - it's returned as a failed `ClaimResult`.
    fn claim_daily_reward(
        &self,
    ) -> impl Future<Output = std::result::Result<ClaimResult, Self::Error>> + Send;
}

/// Type-erased trait for dynamic dispatch of daily reward clients.
///
/// This allows storing different game clients in a single collection.
#[async_trait::async_trait]
pub trait DynDailyRewardClient: Send + Sync {
    /// Returns the game identifier for this client.
    fn game_id(&self) -> GameId;

    /// Gets the reward status as a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if the fetch or serialization fails.
    async fn get_reward_status_json(&self) -> std::result::Result<serde_json::Value, BoxError>;

    /// Claims the daily reward and returns the result as JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if the claim or serialization fails.
    async fn claim_daily_reward_json(&self) -> std::result::Result<serde_json::Value, BoxError>;
}

/// Blanket implementation of `DynDailyRewardClient` for all `DailyRewardClient` implementors.
#[async_trait::async_trait]
impl<T> DynDailyRewardClient for T
where
    T: DailyRewardClient,
{
    fn game_id(&self) -> GameId {
        DailyRewardClient::game_id(self)
    }

    async fn get_reward_status_json(&self) -> std::result::Result<serde_json::Value, BoxError> {
        let status = self
            .get_reward_status()
            .await
            .map_err(|e| Box::new(e) as BoxError)?;
        serde_json::to_value(status).map_err(|e| Box::new(e) as BoxError)
    }

    async fn claim_daily_reward_json(&self) -> std::result::Result<serde_json::Value, BoxError> {
        let result = self
            .claim_daily_reward()
            .await
            .map_err(|e| Box::new(e) as BoxError)?;
        serde_json::to_value(result).map_err(|e| Box::new(e) as BoxError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // DailyRewardInfo tests
    // =========================================================================

    #[test]
    fn test_daily_reward_info_new() {
        let info = DailyRewardInfo::new(true, 15);

        assert!(info.is_signed, "is_signed should be true");
        assert_eq!(info.total_sign_day, 15, "total_sign_day should be 15");
    }

    #[test]
    fn test_daily_reward_info_not_signed() {
        let info = DailyRewardInfo::new(false, 10);

        assert!(!info.is_signed, "is_signed should be false");
        assert_eq!(info.total_sign_day, 10);
    }

    #[test]
    fn test_missed_rewards_calculation() {
        use chrono::{FixedOffset, Utc};

        // Get the current day in UTC+8
        let utc8 = FixedOffset::east_opt(8 * 3600).expect("UTC+8 is valid");
        let now = Utc::now().with_timezone(&utc8);
        let current_day = now.day();

        // If we've signed every day, missed should be 0
        let info = DailyRewardInfo::new(true, current_day);
        assert_eq!(
            info.missed_rewards(),
            0,
            "Should have 0 missed rewards when signed all days"
        );

        // If we've signed no days, missed should be current_day
        let info = DailyRewardInfo::new(false, 0);
        assert_eq!(
            info.missed_rewards(),
            current_day,
            "Should have {current_day} missed rewards when signed 0 days"
        );

        // If we've missed half the days
        if current_day >= 2 {
            let half_days = current_day / 2;
            let info = DailyRewardInfo::new(true, half_days);
            assert_eq!(
                info.missed_rewards(),
                current_day - half_days,
                "Should calculate correct missed rewards"
            );
        }
    }

    #[test]
    fn test_missed_rewards_saturating_sub() {
        // Edge case: total_sign_day > current day (shouldn't happen, but test saturation)
        // This can't actually overflow because we use saturating_sub
        let info = DailyRewardInfo::new(true, 50);
        // If current day is, say, 15, then missed_rewards should be 0 (saturating)
        let missed = info.missed_rewards();
        assert!(
            missed <= 31,
            "missed_rewards should not panic and should be bounded"
        );
    }

    // =========================================================================
    // DailyReward tests
    // =========================================================================

    #[test]
    fn test_daily_reward_new() {
        let reward = DailyReward::new("Primogems", 60, "https://example.com/primogem.png");

        assert_eq!(reward.name, "Primogems");
        assert_eq!(reward.amount, 60);
        assert_eq!(reward.icon, "https://example.com/primogem.png");
    }

    #[test]
    fn test_daily_reward_new_with_string() {
        let name = String::from("Mora");
        let icon = String::from("https://example.com/mora.png");
        let reward = DailyReward::new(name, 10000, icon);

        assert_eq!(reward.name, "Mora");
        assert_eq!(reward.amount, 10000);
    }

    // =========================================================================
    // ClaimResult tests
    // =========================================================================

    #[test]
    fn test_claim_result_success() {
        let reward = DailyReward::new("Primogems", 60, "icon.png");
        let info = DailyRewardInfo::new(true, 15);
        let result = ClaimResult::success(reward, info);

        assert!(result.success, "success flag should be true");
        assert!(result.reward.is_some(), "reward should be present");
        assert!(
            result.message.is_none(),
            "message should be None on success"
        );
        assert!(result.info.is_signed, "info should show signed");
    }

    #[test]
    fn test_claim_result_already_claimed() {
        let reward = DailyReward::new("Primogems", 60, "icon.png");
        let info = DailyRewardInfo::new(true, 15);
        let result = ClaimResult::already_claimed(Some(reward), info);

        assert!(!result.success, "success flag should be false");
        assert!(
            result.reward.is_some(),
            "reward should be present (what would have been claimed)"
        );
        assert!(result.message.is_some(), "message should be present");
        assert_eq!(
            result.message.as_deref(),
            Some("Already claimed today"),
            "message should indicate already claimed"
        );
    }

    #[test]
    fn test_claim_result_already_claimed_no_reward() {
        let info = DailyRewardInfo::new(true, 15);
        let result = ClaimResult::already_claimed(None, info);

        assert!(!result.success);
        assert!(result.reward.is_none(), "reward can be None");
        assert!(result.message.is_some());
    }

    #[test]
    fn test_claim_result_error() {
        let info = DailyRewardInfo::new(false, 10);
        let result = ClaimResult::error("API rate limited", info);

        assert!(!result.success, "success flag should be false on error");
        assert!(result.reward.is_none(), "reward should be None on error");
        assert!(result.message.is_some(), "message should be present");
        assert_eq!(
            result.message.as_deref(),
            Some("API rate limited"),
            "message should contain error text"
        );
    }

    #[test]
    fn test_claim_result_error_with_string() {
        let info = DailyRewardInfo::new(false, 10);
        let error_msg = String::from("Network error");
        let result = ClaimResult::error(error_msg, info);

        assert_eq!(result.message.as_deref(), Some("Network error"));
    }

    // =========================================================================
    // DailyRewardStatus tests
    // =========================================================================

    #[test]
    fn test_daily_reward_status_new() {
        let info = DailyRewardInfo::new(false, 10);
        let today_reward = Some(DailyReward::new("Primogems", 60, "icon.png"));
        let monthly_rewards = vec![
            DailyReward::new("Primogems", 60, "icon.png"),
            DailyReward::new("Mora", 10000, "mora.png"),
        ];

        let status = DailyRewardStatus::new(info.clone(), today_reward, monthly_rewards);

        assert!(!status.info.is_signed);
        assert_eq!(status.info.total_sign_day, 10);
        assert!(status.today_reward.is_some());
        assert_eq!(status.monthly_rewards.len(), 2);
    }

    #[test]
    fn test_daily_reward_status_no_today_reward() {
        let info = DailyRewardInfo::new(true, 15);
        let status = DailyRewardStatus::new(info, None, vec![]);

        assert!(status.info.is_signed);
        assert!(status.today_reward.is_none());
        assert!(status.monthly_rewards.is_empty());
    }

    // =========================================================================
    // Serde tests
    // =========================================================================

    #[test]
    fn test_daily_reward_info_serde_roundtrip() {
        let info = DailyRewardInfo::new(true, 15);

        let json = serde_json::to_string(&info).expect("should serialize");
        let deserialized: DailyRewardInfo =
            serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(info.is_signed, deserialized.is_signed);
        assert_eq!(info.total_sign_day, deserialized.total_sign_day);
    }

    #[test]
    fn test_daily_reward_serde_roundtrip() {
        let reward = DailyReward::new("Primogems", 60, "https://example.com/icon.png");

        let json = serde_json::to_string(&reward).expect("should serialize");
        let deserialized: DailyReward = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(reward.name, deserialized.name);
        assert_eq!(reward.amount, deserialized.amount);
        assert_eq!(reward.icon, deserialized.icon);
    }

    #[test]
    fn test_claim_result_serde_roundtrip() {
        let reward = DailyReward::new("Primogems", 60, "icon.png");
        let info = DailyRewardInfo::new(true, 15);
        let result = ClaimResult::success(reward, info);

        let json = serde_json::to_string(&result).expect("should serialize");
        let deserialized: ClaimResult = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(result.success, deserialized.success);
        assert!(deserialized.reward.is_some());
    }

    #[test]
    fn test_daily_reward_status_serde_roundtrip() {
        let info = DailyRewardInfo::new(false, 10);
        let today_reward = Some(DailyReward::new("Primogems", 60, "icon.png"));
        let monthly_rewards = vec![
            DailyReward::new("Primogems", 60, "icon.png"),
            DailyReward::new("Mora", 10000, "mora.png"),
        ];
        let status = DailyRewardStatus::new(info, today_reward, monthly_rewards);

        let json = serde_json::to_string(&status).expect("should serialize");
        let deserialized: DailyRewardStatus =
            serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(status.info.total_sign_day, deserialized.info.total_sign_day);
        assert_eq!(
            status.monthly_rewards.len(),
            deserialized.monthly_rewards.len()
        );
    }
}
