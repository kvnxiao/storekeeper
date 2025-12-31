//! Genshin Impact game client implementation.

use async_trait::async_trait;
use serde::Deserialize;
use storekeeper_client_hoyolab::HoyolabClient;
use storekeeper_core::{
    CooldownResource, ExpeditionResource, GameClient, GameId, Region, StaminaResource,
};

use crate::error::{Error, Result};
use crate::resource::GenshinResource;

/// Resin regeneration rate: 1 resin per 8 minutes = 480 seconds.
const RESIN_REGEN_SECONDS: u32 = 480;

/// Realm currency regeneration rate varies by trust rank, use approximate.
const REALM_REGEN_SECONDS: u32 = 2400; // Approximate

/// API response structure for Genshin daily note.
#[derive(Debug, Deserialize)]
struct DailyNoteResponse {
    current_resin: u32,
    max_resin: u32,
    resin_recovery_time: String,
    current_home_coin: u32,
    max_home_coin: u32,
    home_coin_recovery_time: String,
    current_expedition_num: u32,
    max_expedition_num: u32,
    expeditions: Vec<ExpeditionInfo>,
    transformer: Option<TransformerInfo>,
}

#[derive(Debug, Deserialize)]
struct ExpeditionInfo {
    remained_time: String,
}

#[derive(Debug, Deserialize)]
struct TransformerInfo {
    obtained: bool,
    recovery_time: TransformerRecoveryTime,
}

#[derive(Debug, Deserialize)]
struct TransformerRecoveryTime {
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
        let resin_seconds = note
            .resin_recovery_time
            .parse::<u64>()
            .ok()
            .filter(|&s| s > 0);
        resources.push(GenshinResource::Resin(StaminaResource::new(
            note.current_resin,
            note.max_resin,
            resin_seconds,
            RESIN_REGEN_SECONDS,
        )));

        // Realm Currency
        let realm_seconds = note
            .home_coin_recovery_time
            .parse::<u64>()
            .ok()
            .filter(|&s| s > 0);
        resources.push(GenshinResource::RealmCurrency(StaminaResource::new(
            note.current_home_coin,
            note.max_home_coin,
            realm_seconds,
            REALM_REGEN_SECONDS,
        )));

        // Parametric Transformer
        if let Some(transformer) = note.transformer {
            if transformer.obtained {
                let cooldown = if transformer.recovery_time.reached {
                    CooldownResource::ready()
                } else {
                    let seconds = u64::from(transformer.recovery_time.day) * 86400
                        + u64::from(transformer.recovery_time.hour) * 3600
                        + u64::from(transformer.recovery_time.minute) * 60
                        + u64::from(transformer.recovery_time.second);
                    CooldownResource::on_cooldown(seconds)
                };
                resources.push(GenshinResource::ParametricTransformer(cooldown));
            }
        }

        // Expeditions
        let earliest = note
            .expeditions
            .iter()
            .filter_map(|e| e.remained_time.parse::<u64>().ok())
            .filter(|&s| s > 0)
            .min();
        resources.push(GenshinResource::Expeditions(ExpeditionResource::new(
            note.current_expedition_num,
            note.max_expedition_num,
            earliest,
        )));

        tracing::debug!(
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
