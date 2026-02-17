//! Game client initialization for the registry.

use storekeeper_client_hoyolab::{
    GENSHIN_DAILY_REWARD, HSR_DAILY_REWARD, HoyolabClient, HoyolabDailyRewardClient,
    ZZZ_DAILY_REWARD,
};
use storekeeper_client_kuro::load_oauth_from_cache;
use storekeeper_core::{AppConfig, DynDailyRewardClient, DynGameClient, Region, SecretsConfig};
use storekeeper_game_genshin::GenshinClient;
use storekeeper_game_hsr::HsrClient;
use storekeeper_game_wuwa::WuwaClient;
use storekeeper_game_zzz::ZzzClient;

use crate::daily_reward_registry::DailyRewardRegistry;
use crate::registry::GameClientRegistry;

/// Registers a HoYoLab-based game client if enabled and region can be resolved.
fn register_hoyolab_game(
    registry: &mut GameClientRegistry,
    hoyolab: &HoyolabClient,
    uid: &str,
    region_override: Option<Region>,
    detect_region: impl FnOnce(&str) -> std::result::Result<Region, storekeeper_core::Error>,
    create_client: impl FnOnce(HoyolabClient, &str, Region) -> Box<dyn DynGameClient>,
    game_name: &str,
) {
    let region = region_override.or_else(|| detect_region(uid).ok());
    if let Some(region) = region {
        let client = create_client(hoyolab.clone(), uid, region);
        tracing::info!(uid = %uid, region = ?region, "{game_name} client registered");
        registry.register(client);
    }
}

/// Creates a `GameClientRegistry` from configuration and secrets.
///
/// HoYoLab-based game clients share a single `HoyolabClient` instance to
/// avoid redundant HTTP client allocations.
#[must_use]
pub fn create_registry(config: &AppConfig, secrets: &SecretsConfig) -> GameClientRegistry {
    tracing::info!("Creating game client registry from configuration");
    let mut registry = GameClientRegistry::new();

    // Initialize HoYoLab-based clients if credentials are configured
    if secrets.hoyolab.is_configured() {
        tracing::debug!("HoYoLab credentials found, initializing HoYoLab-based clients");
        let ltuid = secrets.hoyolab.ltuid();
        let ltoken = secrets.hoyolab.ltoken();

        let hoyolab = match HoyolabClient::new(ltuid, ltoken) {
            Ok(client) => client,
            Err(e) => {
                tracing::warn!("Failed to create shared HoYoLab client: {e}");
                return registry;
            }
        };

        // Genshin Impact
        if let Some(ref c) = config.games.genshin_impact {
            if c.enabled {
                register_hoyolab_game(
                    &mut registry,
                    &hoyolab,
                    &c.uid,
                    c.region,
                    Region::from_genshin_uid,
                    |h, uid, r| Box::new(GenshinClient::new(h, uid, r)),
                    "Genshin Impact",
                );
            }
        }

        // Honkai: Star Rail
        if let Some(ref c) = config.games.honkai_star_rail {
            if c.enabled {
                register_hoyolab_game(
                    &mut registry,
                    &hoyolab,
                    &c.uid,
                    c.region,
                    Region::from_hsr_uid,
                    |h, uid, r| Box::new(HsrClient::new(h, uid, r)),
                    "Honkai: Star Rail",
                );
            }
        }

        // Zenless Zone Zero
        if let Some(ref c) = config.games.zenless_zone_zero {
            if c.enabled {
                register_hoyolab_game(
                    &mut registry,
                    &hoyolab,
                    &c.uid,
                    c.region,
                    Region::from_zzz_uid,
                    |h, uid, r| Box::new(ZzzClient::new(h, uid, r)),
                    "Zenless Zone Zero",
                );
            }
        }
    } else {
        tracing::debug!("HoYoLab credentials not configured, skipping HoYoLab-based clients");
    }

    // Initialize Kuro-based clients (Wuthering Waves)
    if let Some(ref wuwa_config) = config.games.wuthering_waves {
        if wuwa_config.enabled {
            let oauth_code = secrets
                .kuro
                .oauth_code_override()
                .map(String::from)
                .or_else(|| match load_oauth_from_cache() {
                    Ok(code) => code,
                    Err(e) => {
                        tracing::warn!("Failed to load Kuro OAuth code from cache: {e}");
                        None
                    }
                });

            if let Some(oauth_code) = oauth_code {
                let region = wuwa_config
                    .region
                    .or_else(|| Region::from_wuwa_player_id(&wuwa_config.player_id).ok());
                if let Some(region) = region {
                    if let Ok(client) = WuwaClient::new(&oauth_code, &wuwa_config.player_id, region)
                    {
                        tracing::info!(
                            player_id = %wuwa_config.player_id,
                            region = ?region,
                            "Wuthering Waves client registered"
                        );
                        registry.register(Box::new(client) as Box<dyn DynGameClient>);
                    }
                }
            } else {
                tracing::warn!(
                    "Wuthering Waves is enabled but no OAuth code available. \
                     Set oauth_code in secrets.toml or ensure the Kuro launcher cache exists."
                );
            }
        }
    }

    tracing::info!(
        client_count = registry.len(),
        "Game client registry creation complete"
    );

    registry
}

/// Creates a `DailyRewardRegistry` from configuration and secrets.
///
/// Daily reward clients share a single `HoyolabClient` and differ only by
/// their endpoint configuration.
#[must_use]
pub fn create_daily_reward_registry(
    config: &AppConfig,
    secrets: &SecretsConfig,
) -> DailyRewardRegistry {
    tracing::info!("Creating daily reward registry from configuration");
    let mut registry = DailyRewardRegistry::new();

    if !secrets.hoyolab.is_configured() {
        tracing::debug!("HoYoLab credentials not configured, skipping daily reward clients");
        return registry;
    }

    let ltuid = secrets.hoyolab.ltuid();
    let ltoken = secrets.hoyolab.ltoken();

    let hoyolab = match HoyolabClient::new(ltuid, ltoken) {
        Ok(client) => client,
        Err(e) => {
            tracing::warn!("Failed to create HoYoLab client for daily rewards: {e}");
            return registry;
        }
    };

    // Genshin Impact
    if config
        .games
        .genshin_impact
        .as_ref()
        .is_some_and(|c| c.enabled)
    {
        let client = HoyolabDailyRewardClient::new(hoyolab.clone(), &GENSHIN_DAILY_REWARD);
        tracing::info!("Genshin Impact daily reward client registered");
        registry.register(Box::new(client) as Box<dyn DynDailyRewardClient>);
    }

    // Honkai: Star Rail
    if config
        .games
        .honkai_star_rail
        .as_ref()
        .is_some_and(|c| c.enabled)
    {
        let client = HoyolabDailyRewardClient::new(hoyolab.clone(), &HSR_DAILY_REWARD);
        tracing::info!("Honkai: Star Rail daily reward client registered");
        registry.register(Box::new(client) as Box<dyn DynDailyRewardClient>);
    }

    // Zenless Zone Zero
    if config
        .games
        .zenless_zone_zero
        .as_ref()
        .is_some_and(|c| c.enabled)
    {
        let client = HoyolabDailyRewardClient::new(hoyolab, &ZZZ_DAILY_REWARD);
        tracing::info!("Zenless Zone Zero daily reward client registered");
        registry.register(Box::new(client) as Box<dyn DynDailyRewardClient>);
    }

    tracing::info!(
        client_count = registry.len(),
        "Daily reward registry creation complete"
    );

    registry
}
