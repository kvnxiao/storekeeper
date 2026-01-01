//! Zenless Zone Zero game client implementation.

use async_trait::async_trait;
use serde::Deserialize;
use storekeeper_client_hoyolab::{HoyolabClient, Method};
use storekeeper_core::{
    ClaimResult, DailyReward, DailyRewardClient, DailyRewardInfo, DailyRewardStatus, GameClient,
    GameId, Region, StaminaResource,
};

use crate::error::{Error, Result};
use crate::resource::ZzzResource;

/// Battery regeneration rate: 1 battery per 6 minutes = 360 seconds.
const BATTERY_REGEN_SECONDS: u32 = 360;

/// Daily reward base URL for Zenless Zone Zero (overseas).
const ZZZ_REWARD_URL: &str = "https://sg-public-api.hoyolab.com/event/luna/zzz/os";

/// Act ID for ZZZ daily rewards.
const ZZZ_ACT_ID: &str = "e202406031448091";

/// Sign game header value for ZZZ.
const ZZZ_SIGN_GAME: &str = "zzz";

/// API response structure for ZZZ note.
#[derive(Debug, Deserialize)]
struct NoteResponse {
    energy: EnergyInfo,
}

#[derive(Debug, Deserialize)]
struct EnergyInfo {
    progress: EnergyProgress,
    restore: u64,
}

#[derive(Debug, Deserialize)]
struct EnergyProgress {
    current: u32,
    max: u32,
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

/// Zenless Zone Zero game client.
#[derive(Debug, Clone)]
pub struct ZzzClient {
    hoyolab: HoyolabClient,
    uid: String,
    region: Region,
}

impl ZzzClient {
    /// Creates a new ZZZ client.
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
        tracing::debug!(uid = %self.uid, region = ?self.region, "Fetching ZZZ note");
        let url = format!(
            "https://sg-public-api.hoyolab.com/event/game_record_zzz/api/zzz/note?server={}&role_id={}",
            self.region.zzz_region(),
            self.uid
        );

        self.hoyolab.get(&url).await.map_err(Error::HoyolabApi)
    }
}

#[async_trait]
impl GameClient for ZzzClient {
    type Resource = ZzzResource;
    type Error = Error;

    fn game_id(&self) -> GameId {
        GameId::ZenlessZoneZero
    }

    fn game_name(&self) -> &'static str {
        GameId::ZenlessZoneZero.display_name()
    }

    async fn fetch_resources(&self) -> Result<Vec<Self::Resource>> {
        tracing::info!(game = "Zenless Zone Zero", "Fetching game resources");
        let note = self.fetch_note().await?;

        let seconds_until_full = if note.energy.restore > 0 {
            Some(note.energy.restore)
        } else {
            None
        };

        tracing::debug!(
            battery = note.energy.progress.current,
            max_battery = note.energy.progress.max,
            "ZZZ resources fetched successfully"
        );

        Ok(vec![ZzzResource::Battery(StaminaResource::new(
            note.energy.progress.current,
            note.energy.progress.max,
            seconds_until_full,
            BATTERY_REGEN_SECONDS,
        ))])
    }

    async fn is_authenticated(&self) -> Result<bool> {
        self.hoyolab.check_auth().await.map_err(Error::HoyolabApi)
    }
}

// ============================================================================
// Daily Reward Client Implementation
// ============================================================================

impl ZzzClient {
    /// Builds a daily reward URL with the given endpoint.
    fn reward_url(endpoint: &str) -> String {
        format!(
            "{}?act_id={ZZZ_ACT_ID}&lang=en-us",
            ZZZ_REWARD_URL.replace("/os", &format!("/os/{endpoint}"))
        )
    }

    /// Returns the headers required for daily reward requests.
    fn reward_headers() -> [(&'static str, &'static str); 2] {
        [
            ("x-rpc-signgame", ZZZ_SIGN_GAME),
            ("referer", "https://act.hoyolab.com/"),
        ]
    }
}

#[async_trait]
impl DailyRewardClient for ZzzClient {
    type Error = Error;

    fn game_id(&self) -> GameId {
        GameId::ZenlessZoneZero
    }

    async fn get_reward_info(&self) -> Result<DailyRewardInfo> {
        tracing::debug!(game = "Zenless Zone Zero", "Fetching daily reward info");

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
        tracing::debug!(game = "Zenless Zone Zero", "Fetching monthly rewards");

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
        tracing::debug!(game = "Zenless Zone Zero", "Fetching daily reward status");

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
        tracing::info!(game = "Zenless Zone Zero", "Claiming daily reward");

        let pre_info = self.get_reward_info().await?;
        if pre_info.is_signed {
            tracing::debug!(game = "Zenless Zone Zero", "Daily reward already claimed");
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
            game = "Zenless Zone Zero",
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
