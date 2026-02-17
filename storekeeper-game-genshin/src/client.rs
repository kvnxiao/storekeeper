//! Genshin Impact game client implementation.

use chrono::{DateTime, Local, TimeDelta};
use serde::{Deserialize, Deserializer};
use storekeeper_client_hoyolab::HoyolabClient;
use storekeeper_core::{
    CooldownResource, ExpeditionResource, GameClient, GameId, Region, StaminaResource, serde_utils,
};

use crate::error::{Error, Result};
use crate::resource::GenshinResource;

/// Resin regeneration rate: 1 resin per 8 minutes = 480 seconds.
const RESIN_REGEN_SECONDS: u32 = 480;

/// Realm currency regeneration rate varies by trust rank, assume max trust rank
/// "Fit for a King" which is 30 coins per hour.
const REALM_REGEN_SECONDS: u32 = 120;

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

/// Genshin Impact game client.
#[derive(Debug, Clone)]
pub struct GenshinClient {
    hoyolab: HoyolabClient,
    uid: String,
    region: Region,
}

impl GenshinClient {
    /// Creates a new Genshin Impact client using a shared `HoyolabClient`.
    #[must_use]
    pub fn new(hoyolab: HoyolabClient, uid: impl Into<String>, region: Region) -> Self {
        Self {
            hoyolab,
            uid: uid.into(),
            region,
        }
    }

    /// Fetches the daily note data from the API.
    async fn fetch_daily_note(&self) -> Result<DailyNoteResponse> {
        tracing::debug!(uid = %self.uid, region = ?self.region, "Fetching Genshin daily note");
        let url = format!(
            "https://sg-public-api.hoyolab.com/event/game_record/genshin/api/dailyNote?server={}&role_id={}",
            self.region.genshin_region(),
            self.uid
        );

        self.hoyolab.get(&url).await
    }
}

impl GameClient for GenshinClient {
    type Resource = GenshinResource;
    type Error = Error;

    fn game_id(&self) -> GameId {
        GameId::GenshinImpact
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
        self.hoyolab.check_auth().await
    }
}
