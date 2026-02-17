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

fn build_resources(note: &DailyNoteResponse, now: DateTime<Local>) -> Vec<GenshinResource> {
    let mut resources = Vec::with_capacity(4);

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
            let cooldown = CooldownResource::new(transformer.ready_at <= now, transformer.ready_at);
            resources.push(GenshinResource::ParametricTransformer(cooldown));
        }
    }

    // Expeditions - find the earliest finish time
    let earliest_finish = note
        .expeditions
        .iter()
        .map(|expedition| expedition.remained_time)
        .min();
    if earliest_finish.is_none() && note.current_expedition_num > 0 {
        tracing::warn!("Genshin reported active expeditions but expedition list is empty");
    }
    let earliest_finish = earliest_finish.unwrap_or(now);
    resources.push(GenshinResource::Expeditions(ExpeditionResource::new(
        note.current_expedition_num,
        note.max_expedition_num,
        earliest_finish,
    )));

    resources
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
        let resources = build_resources(&note, Local::now());

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

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use serde_json::json;

    use super::*;

    fn make_note(
        transformer: Option<TransformerInfo>,
        expeditions: Vec<ExpeditionInfo>,
        current_expeditions: u32,
    ) -> DailyNoteResponse {
        let now = Local::now();
        DailyNoteResponse {
            current_resin: 100,
            max_resin: 160,
            resin_recovery_time: now + TimeDelta::try_hours(8).expect("valid delta"),
            current_home_coin: 1200,
            max_home_coin: 2400,
            home_coin_recovery_time: now + TimeDelta::try_hours(4).expect("valid delta"),
            current_expedition_num: current_expeditions,
            max_expedition_num: 5,
            expeditions,
            transformer,
        }
    }

    #[test]
    fn transformer_deserialize_reached_sets_ready_at_now() {
        let before = Local::now();
        let transformer: TransformerInfo = serde_json::from_value(json!({
            "obtained": true,
            "recovery_time": {
                "Day": 0,
                "Hour": 0,
                "Minute": 0,
                "Second": 0,
                "reached": true
            }
        }))
        .expect("deserialize transformer");
        let after = Local::now();

        assert!(transformer.obtained);
        assert!(transformer.ready_at >= before && transformer.ready_at <= after);
    }

    #[test]
    fn transformer_deserialize_positive_duration_adds_delta() {
        let transformer: TransformerInfo = serde_json::from_value(json!({
            "obtained": true,
            "recovery_time": {
                "Day": 0,
                "Hour": 0,
                "Minute": 1,
                "Second": 30,
                "reached": false
            }
        }))
        .expect("deserialize transformer");

        let seconds_until_ready = (transformer.ready_at - Local::now()).num_seconds();
        assert!(
            (85..=95).contains(&seconds_until_ready),
            "Expected ~90s until ready, got {seconds_until_ready}s"
        );
    }

    #[test]
    fn transformer_deserialize_zero_duration_falls_back_to_now() {
        let before = Local::now();
        let transformer: TransformerInfo = serde_json::from_value(json!({
            "obtained": true,
            "recovery_time": {
                "Day": 0,
                "Hour": 0,
                "Minute": 0,
                "Second": 0,
                "reached": false
            }
        }))
        .expect("deserialize transformer");
        let after = Local::now();

        assert!(transformer.ready_at >= before && transformer.ready_at <= after);
    }

    #[test]
    fn build_resources_omits_transformer_when_not_obtained() {
        let now = Local::now();
        let note = make_note(
            Some(TransformerInfo {
                obtained: false,
                ready_at: now + TimeDelta::try_hours(10).expect("valid delta"),
            }),
            vec![ExpeditionInfo {
                remained_time: now + TimeDelta::try_minutes(20).expect("valid delta"),
                avatar_side_icon: "icon".to_string(),
                status: ExpeditionStatus::Ongoing,
            }],
            1,
        );

        let resources = build_resources(&note, now);
        let has_transformer = resources
            .iter()
            .any(|resource| matches!(resource, GenshinResource::ParametricTransformer(_)));
        assert!(
            !has_transformer,
            "Not-obtained transformer should be excluded"
        );
    }

    #[test]
    fn build_resources_uses_now_when_expeditions_are_missing() {
        let now = Local::now();
        let note = make_note(None, Vec::new(), 2);
        let resources = build_resources(&note, now);

        let expedition = resources.iter().find_map(|resource| match resource {
            GenshinResource::Expeditions(expedition) => Some(expedition),
            _ => None,
        });

        assert!(
            expedition.is_some(),
            "Expeditions resource should always be present"
        );
        let expedition = expedition.expect("expedition resource should exist");
        assert_eq!(expedition.current_expeditions, 2);
        assert_eq!(expedition.max_expeditions, 5);
        assert_eq!(
            expedition.earliest_finish_at, now,
            "When expedition list is empty, fallback should use provided current time"
        );
    }
}
