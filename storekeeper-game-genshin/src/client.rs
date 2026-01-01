//! Genshin Impact game client implementation.

use async_trait::async_trait;
use chrono::{DateTime, Local, TimeDelta};
use serde::{Deserialize, Deserializer};
use storekeeper_client_hoyolab::{HoyolabClient, Method};
use storekeeper_core::{
    ClaimResult, CooldownResource, DailyReward, DailyRewardClient, DailyRewardInfo,
    DailyRewardStatus, ExpeditionResource, GameClient, GameId, Region, StaminaResource,
    serde_utils,
};

use crate::error::{Error, Result};
use crate::resource::GenshinResource;

/// Resin regeneration rate: 1 resin per 8 minutes = 480 seconds.
const RESIN_REGEN_SECONDS: u32 = 480;

/// Realm currency regeneration rate varies by trust rank, assume max trust rank
/// "Fit for a King" which is 30 coins per hour.
const REALM_REGEN_SECONDS: u32 = 120;

/// Daily reward base URL for Genshin Impact (overseas).
const GENSHIN_REWARD_URL: &str = "https://sg-hk4e-api.hoyolab.com/event/sol";

/// Act ID for Genshin Impact daily rewards.
const GENSHIN_ACT_ID: &str = "e202102251931481";

/// Sign game header value for Genshin Impact.
const GENSHIN_SIGN_GAME: &str = "hk4e";

// ============================================================================
// Daily Note API Response Structures
// ============================================================================

/// API response structure for Genshin daily note.
#[derive(Debug, Clone, Deserialize)]
struct DailyNoteResponse {
    current_resin: u32,
    max_resin: u32,
    #[serde(deserialize_with = "serde_utils::seconds_string_to_datetime::deserialize")]
    resin_recovery_time: DateTime<Local>,
    current_home_coin: u32,
    max_home_coin: u32,
    #[serde(deserialize_with = "serde_utils::seconds_string_to_datetime::deserialize")]
    home_coin_recovery_time: DateTime<Local>,
    current_expedition_num: u32,
    max_expedition_num: u32,
    expeditions: Vec<ExpeditionInfo>,
    transformer: Option<TransformerInfo>,
}

/// Status of an expedition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ExpeditionStatus {
    /// Expedition is still in progress.
    Ongoing,
    /// Expedition has finished and can be collected.
    Finished,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpeditionInfo {
    #[serde(deserialize_with = "serde_utils::seconds_string_to_datetime::deserialize")]
    remained_time: DateTime<Local>,
    /// URL to the avatar's side icon.
    #[allow(dead_code)]
    avatar_side_icon: String,
    /// Current status of the expedition.
    #[allow(dead_code)]
    status: ExpeditionStatus,
}

/// Transformer information with custom deserialization.
#[derive(Debug, Clone)]
struct TransformerInfo {
    obtained: bool,
    ready_at: DateTime<Local>,
}

impl<'de> Deserialize<'de> for TransformerInfo {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// Raw API structure for transformer recovery time.
        #[derive(Deserialize)]
        struct RawRecoveryTime {
            #[serde(rename = "Day")]
            day: u32,
            #[serde(rename = "Hour")]
            hour: u32,
            #[serde(rename = "Minute")]
            minute: u32,
            #[serde(rename = "Second")]
            second: u32,
            reached: bool,
        }

        /// Raw API structure for transformer info.
        #[derive(Deserialize)]
        struct RawTransformerInfo {
            obtained: bool,
            recovery_time: RawRecoveryTime,
        }

        let raw = RawTransformerInfo::deserialize(deserializer)?;

        let ready_at = if raw.recovery_time.reached {
            Local::now()
        } else {
            let total_seconds = i64::from(raw.recovery_time.day) * 86400
                + i64::from(raw.recovery_time.hour) * 3600
                + i64::from(raw.recovery_time.minute) * 60
                + i64::from(raw.recovery_time.second);

            if total_seconds > 0 {
                TimeDelta::try_seconds(total_seconds)
                    .map_or_else(Local::now, |delta| Local::now() + delta)
            } else {
                Local::now()
            }
        };

        Ok(TransformerInfo {
            obtained: raw.obtained,
            ready_at,
        })
    }
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

/// Genshin Impact game client.
#[derive(Debug, Clone)]
pub struct GenshinClient {
    hoyolab: HoyolabClient,
    uid: String,
    region: Region,
}

impl GenshinClient {
    /// Creates a new Genshin Impact client.
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

    /// Fetches the daily note data from the API.
    async fn fetch_daily_note(&self) -> Result<DailyNoteResponse> {
        tracing::debug!(uid = %self.uid, region = ?self.region, "Fetching Genshin daily note");
        let url = format!(
            "https://sg-public-api.hoyolab.com/event/game_record/genshin/api/dailyNote?server={}&role_id={}",
            self.region.genshin_region(),
            self.uid
        );

        self.hoyolab.get(&url).await.map_err(Error::HoyolabApi)
    }
}

#[async_trait]
impl GameClient for GenshinClient {
    type Resource = GenshinResource;
    type Error = Error;

    fn game_id(&self) -> GameId {
        GameId::GenshinImpact
    }

    fn game_name(&self) -> &'static str {
        GameId::GenshinImpact.display_name()
    }

    async fn fetch_resources(&self) -> Result<Vec<Self::Resource>> {
        tracing::info!(game = "Genshin Impact", "Fetching game resources");
        let note = self.fetch_daily_note().await?;
        let mut resources = Vec::new();

        // Resin
        resources.push(GenshinResource::Resin(StaminaResource::new(
            note.current_resin,
            note.max_resin,
            note.resin_recovery_time,
            RESIN_REGEN_SECONDS,
        )));

        // Realm Currency
        resources.push(GenshinResource::RealmCurrency(StaminaResource::new(
            note.current_home_coin,
            note.max_home_coin,
            note.home_coin_recovery_time,
            REALM_REGEN_SECONDS,
        )));

        // Parametric Transformer
        if let Some(ref transformer) = note.transformer {
            if transformer.obtained {
                let cooldown = CooldownResource::new(
                    transformer.ready_at <= Local::now(),
                    transformer.ready_at,
                );
                resources.push(GenshinResource::ParametricTransformer(cooldown));
            }
        }

        // Expeditions - find the earliest finish time
        let earliest_finish = note
            .expeditions
            .iter()
            .map(|e| e.remained_time)
            .min()
            .unwrap_or_else(Local::now);
        resources.push(GenshinResource::Expeditions(ExpeditionResource::new(
            note.current_expedition_num,
            note.max_expedition_num,
            earliest_finish,
        )));

        tracing::info!(
            resin = note.current_resin,
            max_resin = note.max_resin,
            realm_currency = note.current_home_coin,
            expeditions = note.current_expedition_num,
            "Genshin resources fetched successfully"
        );

        Ok(resources)
    }

    async fn is_authenticated(&self) -> Result<bool> {
        self.hoyolab.check_auth().await.map_err(Error::HoyolabApi)
    }
}

// ============================================================================
// Daily Reward Client Implementation
// ============================================================================

impl GenshinClient {
    /// Builds a daily reward URL with the given endpoint.
    fn reward_url(endpoint: &str) -> String {
        format!("{GENSHIN_REWARD_URL}/{endpoint}?act_id={GENSHIN_ACT_ID}&lang=en-us")
    }

    /// Returns the headers required for daily reward requests.
    fn reward_headers() -> [(&'static str, &'static str); 2] {
        [
            ("x-rpc-signgame", GENSHIN_SIGN_GAME),
            ("referer", "https://act.hoyolab.com/"),
        ]
    }
}

#[async_trait]
impl DailyRewardClient for GenshinClient {
    type Error = Error;

    fn game_id(&self) -> GameId {
        GameId::GenshinImpact
    }

    async fn get_reward_info(&self) -> Result<DailyRewardInfo> {
        tracing::debug!(game = "Genshin Impact", "Fetching daily reward info");

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
        tracing::debug!(game = "Genshin Impact", "Fetching monthly rewards");

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
        tracing::debug!(game = "Genshin Impact", "Fetching daily reward status");

        // Fetch info and rewards concurrently
        let (info, rewards) = tokio::try_join!(self.get_reward_info(), self.get_monthly_rewards())?;

        // Determine today's reward index
        // If already signed: total_sign_day - 1 (0-indexed, what was claimed)
        // If not signed: total_sign_day (what will be claimed next)
        let today_index = if info.is_signed {
            info.total_sign_day.saturating_sub(1) as usize
        } else {
            info.total_sign_day as usize
        };

        let today_reward = rewards.get(today_index).cloned();

        Ok(DailyRewardStatus::new(info, today_reward, rewards))
    }

    async fn claim_daily_reward(&self) -> Result<ClaimResult> {
        tracing::info!(game = "Genshin Impact", "Claiming daily reward");

        // Check current status first
        let pre_info = self.get_reward_info().await?;
        if pre_info.is_signed {
            tracing::debug!(game = "Genshin Impact", "Daily reward already claimed");
            let status = self.get_reward_status().await?;
            return Ok(ClaimResult::already_claimed(
                status.today_reward,
                status.info,
            ));
        }

        // Perform the claim
        let url = Self::reward_url("sign");
        let headers = Self::reward_headers();

        let _: serde_json::Value = self
            .hoyolab
            .request_with_headers::<serde_json::Value, ()>(Method::POST, &url, None, &headers)
            .await
            .map_err(Error::HoyolabApi)?;

        // Fetch updated status to get reward details
        let status = self.get_reward_status().await?;

        tracing::info!(
            game = "Genshin Impact",
            reward_name = ?status.today_reward.as_ref().map_or("Unknown", |r| r.name.as_str()),
            "Daily reward claimed successfully"
        );

        // The today_reward now points to what was just claimed
        match status.today_reward {
            Some(reward) => Ok(ClaimResult::success(reward, status.info)),
            None => Ok(ClaimResult::error(
                "Claim succeeded but reward details unavailable",
                status.info,
            )),
        }
    }
}
