//! Wuthering Waves game client implementation.

use async_trait::async_trait;
use serde::Deserialize;
use storekeeper_client_kuro::KuroClient;
use storekeeper_core::{GameClient, GameId, Region, StaminaResource};

use crate::error::{Error, Result};
use crate::resource::WuwaResource;

/// Waveplate regeneration rate: 1 waveplate per 6 minutes = 360 seconds.
const WAVEPLATE_REGEN_SECONDS: u32 = 360;

/// API response structure for WuWa role data.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RoleDataResponse {
    base: BaseInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BaseInfo {
    energy: u32,
    max_energy: u32,
    energy_recover_time: u64,
}

/// Wuthering Waves game client.
#[derive(Debug, Clone)]
pub struct WuwaClient {
    kuro: KuroClient,
    player_id: String,
    region: Region,
}

impl WuwaClient {
    /// Creates a new WuWa client.
    ///
    /// # Errors
    ///
    /// Returns an error if the Kuro client cannot be created.
    pub fn new(
        oauth_code: impl Into<String>,
        player_id: impl Into<String>,
        region: Region,
    ) -> Result<Self> {
        let kuro = KuroClient::new(oauth_code)?;
        Ok(Self {
            kuro,
            player_id: player_id.into(),
            region,
        })
    }

    /// Fetches the role data from the API.
    async fn fetch_role_data(&self) -> Result<RoleDataResponse> {
        tracing::debug!(
            player_id = %self.player_id,
            region = ?self.region,
            "Fetching WuWa role data"
        );
        self.kuro
            .query_role(&self.player_id, self.region.wuwa_region())
            .await
            .map_err(Error::KuroApi)
    }
}

#[async_trait]
impl GameClient for WuwaClient {
    type Resource = WuwaResource;
    type Error = Error;

    fn game_id(&self) -> GameId {
        GameId::WutheringWaves
    }

    fn game_name(&self) -> &'static str {
        GameId::WutheringWaves.display_name()
    }

    async fn fetch_resources(&self) -> Result<Vec<Self::Resource>> {
        tracing::info!(game = "Wuthering Waves", "Fetching game resources");
        let data = self.fetch_role_data().await?;

        // Calculate seconds until full from the recover timestamp
        // timestamp_millis() can technically be negative before Unix epoch, but we're always in the future
        #[allow(clippy::cast_sign_loss)]
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let seconds_until_full = if data.base.energy_recover_time > now_ms {
            Some((data.base.energy_recover_time - now_ms) / 1000)
        } else {
            None
        };

        tracing::debug!(
            waveplates = data.base.energy,
            max_waveplates = data.base.max_energy,
            "WuWa resources fetched successfully"
        );

        Ok(vec![WuwaResource::Waveplates(StaminaResource::new(
            data.base.energy,
            data.base.max_energy,
            seconds_until_full,
            WAVEPLATE_REGEN_SECONDS,
        ))])
    }

    async fn is_authenticated(&self) -> Result<bool> {
        self.kuro
            .check_auth(&self.player_id, self.region.wuwa_region())
            .await
            .map_err(Error::KuroApi)
    }
}
