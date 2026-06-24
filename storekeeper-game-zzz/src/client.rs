//! Zenless Zone Zero game client implementation.

use crate::error::Error;
use crate::error::Result;
use crate::resource::ZzzResource;
use jiff::Timestamp;
use serde::Deserialize;
use storekeeper_client_hoyolab::HoyolabClient;
use storekeeper_core::GameClient;
use storekeeper_core::GameId;
use storekeeper_core::Region;
use storekeeper_core::StaminaResource;
use storekeeper_core::serde_utils;

/// Battery regeneration rate: 1 battery per 6 minutes = 360 seconds.
const BATTERY_REGEN_SECONDS: u32 = 360;

/// API response structure for ZZZ note.
#[derive(Debug, Deserialize)]
struct NoteResponse {
    energy: EnergyInfo,
}

#[derive(Debug, Deserialize)]
struct EnergyInfo {
    progress: EnergyProgress,
    #[serde(deserialize_with = "serde_utils::seconds_u64_to_datetime::deserialize")]
    restore: Timestamp,
}

#[derive(Debug, Deserialize)]
struct EnergyProgress {
    current: u32,
    max: u32,
}

/// Zenless Zone Zero game client.
#[derive(Debug, Clone)]
pub struct ZzzClient {
    hoyolab: HoyolabClient,
    uid: String,
    region: Region,
}

impl ZzzClient {
    /// Creates a new ZZZ client using a shared `HoyolabClient`.
    #[must_use]
    pub fn new(hoyolab: HoyolabClient, uid: impl Into<String>, region: Region) -> Self {
        Self {
            hoyolab,
            uid: uid.into(),
            region,
        }
    }

    /// Fetches the note data from the API.
    async fn fetch_note(&self) -> Result<NoteResponse> {
        tracing::debug!(uid = %self.uid, region = ?self.region, "Fetching ZZZ note");
        let url = format!(
            "https://sg-public-api.hoyolab.com/event/game_record_zzz/api/zzz/note?server={}&role_id={}",
            self.region.zzz_region(),
            self.uid
        );

        self.hoyolab.get(&url).await
    }
}

impl GameClient for ZzzClient {
    type Resource = ZzzResource;
    type Error = Error;

    fn game_id(&self) -> GameId {
        GameId::ZenlessZoneZero
    }

    async fn fetch_resources(&self) -> Result<Vec<Self::Resource>> {
        tracing::info!(game = "Zenless Zone Zero", "Fetching game resources");
        let note = self.fetch_note().await?;

        tracing::info!(
            battery = note.energy.progress.current,
            max_battery = note.energy.progress.max,
            "ZZZ resources fetched successfully"
        );

        Ok(vec![ZzzResource::Battery(StaminaResource::new(
            note.energy.progress.current,
            note.energy.progress.max,
            note.energy.restore,
            BATTERY_REGEN_SECONDS,
        ))])
    }

    async fn is_authenticated(&self) -> Result<bool> {
        self.hoyolab.check_auth().await
    }
}
