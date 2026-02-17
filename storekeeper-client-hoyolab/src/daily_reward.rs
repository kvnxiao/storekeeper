//! Generic HoYoLab daily reward client.
//!
//! Provides a config-driven `DailyRewardClient` implementation that works for
//! all HoYoLab games (Genshin Impact, Honkai: Star Rail, Zenless Zone Zero).

use async_trait::async_trait;
use reqwest::Method;
use serde::Deserialize;
use storekeeper_core::{
    ClaimResult, DailyReward, DailyRewardClient, DailyRewardInfo, DailyRewardStatus, GameId,
};

use crate::client::HoyolabClient;
use crate::error::{Error, Result};

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for a HoYoLab daily reward endpoint.
#[derive(Debug, Clone, Copy)]
pub struct HoyolabDailyRewardConfig {
    /// Base URL for the reward API (e.g., `https://sg-hk4e-api.hoyolab.com/event/sol`).
    pub reward_url: &'static str,
    /// Act ID for the daily reward event.
    pub act_id: &'static str,
    /// Value for the `x-rpc-signgame` header.
    pub sign_game: &'static str,
    /// Game identifier.
    pub game_id: GameId,
}

/// Genshin Impact daily reward configuration.
pub const GENSHIN_DAILY_REWARD: HoyolabDailyRewardConfig = HoyolabDailyRewardConfig {
    reward_url: "https://sg-hk4e-api.hoyolab.com/event/sol",
    act_id: "e202102251931481",
    sign_game: "hk4e",
    game_id: GameId::GenshinImpact,
};

/// Honkai: Star Rail daily reward configuration.
pub const HSR_DAILY_REWARD: HoyolabDailyRewardConfig = HoyolabDailyRewardConfig {
    reward_url: "https://sg-public-api.hoyolab.com/event/luna/hkrpg/os",
    act_id: "e202303301540311",
    sign_game: "hkrpg",
    game_id: GameId::HonkaiStarRail,
};

/// Zenless Zone Zero daily reward configuration.
pub const ZZZ_DAILY_REWARD: HoyolabDailyRewardConfig = HoyolabDailyRewardConfig {
    reward_url: "https://sg-public-api.hoyolab.com/event/luna/zzz/os",
    act_id: "e202406031448091",
    sign_game: "zzz",
    game_id: GameId::ZenlessZoneZero,
};

// ============================================================================
// Response Structures
// ============================================================================

/// API response for daily reward info (`/info` endpoint).
#[derive(Debug, Deserialize)]
struct RewardInfoResponse {
    is_sign: bool,
    total_sign_day: u32,
}

/// API response for monthly rewards list (`/home` endpoint).
#[derive(Debug, Deserialize)]
struct RewardHomeResponse {
    awards: Vec<RewardItem>,
}

/// Individual reward item in the monthly rewards list.
#[derive(Debug, Deserialize)]
struct RewardItem {
    name: String,
    #[serde(alias = "cnt")]
    count: u32,
    icon: String,
}

// ============================================================================
// Client
// ============================================================================

/// Generic HoYoLab daily reward client.
pub struct HoyolabDailyRewardClient {
    client: HoyolabClient,
    config: &'static HoyolabDailyRewardConfig,
}

impl HoyolabDailyRewardClient {
    /// Creates a new daily reward client with the given HoYoLab client and config.
    #[must_use]
    pub fn new(client: HoyolabClient, config: &'static HoyolabDailyRewardConfig) -> Self {
        Self { client, config }
    }

    /// Builds a daily reward URL with the given endpoint.
    fn reward_url(&self, endpoint: &str) -> String {
        format!(
            "{}/{}?act_id={}&lang=en-us",
            self.config.reward_url, endpoint, self.config.act_id
        )
    }

    /// Returns the headers required for daily reward requests.
    fn reward_headers(&self) -> [(&'static str, &'static str); 2] {
        [
            ("x-rpc-signgame", self.config.sign_game),
            ("referer", "https://act.hoyolab.com/"),
        ]
    }
}

#[async_trait]
impl DailyRewardClient for HoyolabDailyRewardClient {
    type Error = Error;

    fn game_id(&self) -> GameId {
        self.config.game_id
    }

    async fn get_reward_info(&self) -> Result<DailyRewardInfo> {
        let game = self.config.game_id.display_name();
        tracing::debug!(game = game, "Fetching daily reward info");

        let url = self.reward_url("info");
        let headers = self.reward_headers();

        let response: RewardInfoResponse = self
            .client
            .request_with_headers::<RewardInfoResponse, ()>(Method::GET, &url, None, &headers)
            .await?;

        Ok(DailyRewardInfo::new(
            response.is_sign,
            response.total_sign_day,
        ))
    }

    async fn get_monthly_rewards(&self) -> Result<Vec<DailyReward>> {
        let game = self.config.game_id.display_name();
        tracing::debug!(game = game, "Fetching monthly rewards");

        let url = self.reward_url("home");
        let headers = self.reward_headers();

        let response: RewardHomeResponse = self
            .client
            .request_with_headers::<RewardHomeResponse, ()>(Method::GET, &url, None, &headers)
            .await?;

        let rewards = response
            .awards
            .into_iter()
            .map(|item| DailyReward::new(item.name, item.count, item.icon))
            .collect();

        Ok(rewards)
    }

    async fn get_reward_status(&self) -> Result<DailyRewardStatus> {
        let game = self.config.game_id.display_name();
        tracing::debug!(game = game, "Fetching daily reward status");

        let (info, rewards) = tokio::try_join!(self.get_reward_info(), self.get_monthly_rewards())?;

        let today_index = if info.is_signed {
            info.total_sign_day.saturating_sub(1) as usize
        } else {
            info.total_sign_day as usize
        };

        let today_reward = rewards.get(today_index).cloned();

        Ok(DailyRewardStatus::new(info, today_reward, rewards))
    }

    async fn claim_daily_reward(&self) -> Result<ClaimResult> {
        let game = self.config.game_id.display_name();
        tracing::info!(game = game, "Claiming daily reward");

        // Check current status first
        let pre_info = self.get_reward_info().await?;
        if pre_info.is_signed {
            tracing::debug!(game = game, "Daily reward already claimed");
            let status = self.get_reward_status().await?;
            return Ok(ClaimResult::already_claimed(
                status.today_reward,
                status.info,
            ));
        }

        // Perform the claim
        let url = self.reward_url("sign");
        let headers = self.reward_headers();

        let _ = self
            .client
            .request_with_headers::<serde_json::Value, ()>(Method::POST, &url, None, &headers)
            .await?;

        // Fetch updated status to get reward details
        let status = self.get_reward_status().await?;

        tracing::info!(
            game = game,
            reward_name = ?status.today_reward.as_ref().map_or("Unknown", |r| r.name.as_str()),
            "Daily reward claimed successfully"
        );

        match status.today_reward {
            Some(reward) => Ok(ClaimResult::success(reward, status.info)),
            None => Ok(ClaimResult::error(
                "Claim succeeded but reward details unavailable",
                status.info,
            )),
        }
    }
}
