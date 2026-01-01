//! Honkai: Star Rail game client implementation.

use async_trait::async_trait;
use serde::Deserialize;
use storekeeper_client_hoyolab::{HoyolabClient, Method};
use storekeeper_core::{
    ClaimResult, DailyReward, DailyRewardClient, DailyRewardInfo, DailyRewardStatus, GameClient,
    GameId, Region, StaminaResource,
};

use crate::error::{Error, Result};
use crate::resource::HsrResource;

/// Trailblaze Power regeneration rate: 1 power per 6 minutes = 360 seconds.
const POWER_REGEN_SECONDS: u32 = 360;

/// Daily reward base URL for Honkai: Star Rail (overseas).
const HSR_REWARD_URL: &str = "https://sg-public-api.hoyolab.com/event/luna/hkrpg/os";

/// Act ID for HSR daily rewards.
const HSR_ACT_ID: &str = "e202303301540311";

/// Sign game header value for HSR.
const HSR_SIGN_GAME: &str = "hkrpg";

/// API response structure for HSR note.
#[derive(Debug, Deserialize)]
struct NoteResponse {
    current_stamina: u32,
    max_stamina: u32,
    stamina_recover_time: u64,
}

// ============================================================================
// Daily Reward API Response Structures
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

/// Honkai: Star Rail game client.
#[derive(Debug, Clone)]
pub struct HsrClient {
    hoyolab: HoyolabClient,
    uid: String,
    region: Region,
}

impl HsrClient {
    /// Creates a new HSR client.
    ///
    /// # Errors
    ///
    /// Returns an error if the HoYoLab client cannot be created.
    pub fn new(
        ltuid: impl Into<String>,
        ltoken: impl Into<String>,
        uid: impl Into<String>,
        region: Region,
    ) -> Result<Self> {
        let hoyolab = HoyolabClient::new(ltuid, ltoken)?;
        Ok(Self {
            hoyolab,
            uid: uid.into(),
            region,
        })
    }

    /// Fetches the note data from the API.
    async fn fetch_note(&self) -> Result<NoteResponse> {
        tracing::debug!(uid = %self.uid, region = ?self.region, "Fetching HSR note");
        let url = format!(
            "https://bbs-api-os.hoyolab.com/game_record/hkrpg/api/note?server={}&role_id={}",
            self.region.hsr_region(),
            self.uid
        );

        self.hoyolab.get(&url).await.map_err(Error::HoyolabApi)
    }
}

#[async_trait]
impl GameClient for HsrClient {
    type Resource = HsrResource;
    type Error = Error;

    fn game_id(&self) -> GameId {
        GameId::HonkaiStarRail
    }

    fn game_name(&self) -> &'static str {
        GameId::HonkaiStarRail.display_name()
    }

    async fn fetch_resources(&self) -> Result<Vec<Self::Resource>> {
        tracing::info!(game = "Honkai: Star Rail", "Fetching game resources");
        let note = self.fetch_note().await?;

        let seconds_until_full = if note.stamina_recover_time > 0 {
            Some(note.stamina_recover_time)
        } else {
            None
        };

        tracing::debug!(
            trailblaze_power = note.current_stamina,
            max_power = note.max_stamina,
            "HSR resources fetched successfully"
        );

        Ok(vec![HsrResource::TrailblazePower(StaminaResource::new(
            note.current_stamina,
            note.max_stamina,
            seconds_until_full,
            POWER_REGEN_SECONDS,
        ))])
    }

    async fn is_authenticated(&self) -> Result<bool> {
        self.hoyolab.check_auth().await.map_err(Error::HoyolabApi)
    }
}

// ============================================================================
// Daily Reward Client Implementation
// ============================================================================

impl HsrClient {
    /// Builds a daily reward URL with the given endpoint.
    fn reward_url(endpoint: &str) -> String {
        format!(
            "{}?act_id={HSR_ACT_ID}&lang=en-us",
            HSR_REWARD_URL.replace("/os", &format!("/os/{endpoint}"))
        )
    }

    /// Returns the headers required for daily reward requests.
    fn reward_headers() -> [(&'static str, &'static str); 2] {
        [
            ("x-rpc-signgame", HSR_SIGN_GAME),
            ("referer", "https://act.hoyolab.com/"),
        ]
    }
}

#[async_trait]
impl DailyRewardClient for HsrClient {
    type Error = Error;

    fn game_id(&self) -> GameId {
        GameId::HonkaiStarRail
    }

    async fn get_reward_info(&self) -> Result<DailyRewardInfo> {
        tracing::debug!(game = "Honkai: Star Rail", "Fetching daily reward info");

        let url = Self::reward_url("info");
        let headers = Self::reward_headers();

        let response: RewardInfoResponse = self
            .hoyolab
            .request_with_headers::<RewardInfoResponse, ()>(Method::GET, &url, None, &headers)
            .await
            .map_err(Error::HoyolabApi)?;

        Ok(DailyRewardInfo::new(
            response.is_sign,
            response.total_sign_day,
        ))
    }

    async fn get_monthly_rewards(&self) -> Result<Vec<DailyReward>> {
        tracing::debug!(game = "Honkai: Star Rail", "Fetching monthly rewards");

        let url = Self::reward_url("home");
        let headers = Self::reward_headers();

        let response: RewardHomeResponse = self
            .hoyolab
            .request_with_headers::<RewardHomeResponse, ()>(Method::GET, &url, None, &headers)
            .await
            .map_err(Error::HoyolabApi)?;

        let rewards = response
            .awards
            .into_iter()
            .map(|item| DailyReward::new(item.name, item.count, item.icon))
            .collect();

        Ok(rewards)
    }

    async fn get_reward_status(&self) -> Result<DailyRewardStatus> {
        tracing::debug!(game = "Honkai: Star Rail", "Fetching daily reward status");

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
        tracing::info!(game = "Honkai: Star Rail", "Claiming daily reward");

        let pre_info = self.get_reward_info().await?;
        if pre_info.is_signed {
            tracing::debug!(game = "Honkai: Star Rail", "Daily reward already claimed");
            let status = self.get_reward_status().await?;
            return Ok(ClaimResult::already_claimed(
                status.today_reward,
                status.info,
            ));
        }

        let url = Self::reward_url("sign");
        let headers = Self::reward_headers();

        let _: serde_json::Value = self
            .hoyolab
            .request_with_headers::<serde_json::Value, ()>(Method::POST, &url, None, &headers)
            .await
            .map_err(Error::HoyolabApi)?;

        let status = self.get_reward_status().await?;

        tracing::info!(
            game = "Honkai: Star Rail",
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
