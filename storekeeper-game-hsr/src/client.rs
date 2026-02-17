//! Honkai: Star Rail game client implementation.

use async_trait::async_trait;
use chrono::{DateTime, Local};
use serde::Deserialize;
use storekeeper_client_hoyolab::HoyolabClient;
use storekeeper_core::{GameClient, GameId, Region, StaminaResource, serde_utils};

use crate::error::{Error, Result};
use crate::resource::HsrResource;

/// Trailblaze Power regeneration rate: 1 power per 6 minutes = 360 seconds.
const POWER_REGEN_SECONDS: u32 = 360;

/// API response structure for HSR note.
#[derive(Debug, Deserialize)]
struct NoteResponse {
    current_stamina: u32,
    max_stamina: u32,
    #[serde(deserialize_with = "serde_utils::seconds_u64_to_datetime::deserialize")]
    stamina_recover_time: DateTime<Local>,
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

        Ok(self.hoyolab.get(&url).await?)
    }
}

#[async_trait]
impl GameClient for HsrClient {
    type Resource = HsrResource;
    type Error = Error;

    fn game_id(&self) -> GameId {
        GameId::HonkaiStarRail
    }

    async fn fetch_resources(&self) -> Result<Vec<Self::Resource>> {
        tracing::info!(game = "Honkai: Star Rail", "Fetching game resources");
        let note = self.fetch_note().await?;

        tracing::info!(
            trailblaze_power = note.current_stamina,
            max_power = note.max_stamina,
            "HSR resources fetched successfully"
        );

        Ok(vec![HsrResource::TrailblazePower(StaminaResource::new(
            note.current_stamina,
            note.max_stamina,
            note.stamina_recover_time,
            POWER_REGEN_SECONDS,
        ))])
    }

    async fn is_authenticated(&self) -> Result<bool> {
        Ok(self.hoyolab.check_auth().await?)
    }
}
