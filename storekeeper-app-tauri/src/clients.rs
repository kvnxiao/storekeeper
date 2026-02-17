//! Game client initialization for the registry.

use storekeeper_client_hoyolab::{
    GENSHIN_DAILY_REWARD, HSR_DAILY_REWARD, HoyolabClient, HoyolabDailyRewardClient,
    HoyolabDailyRewardConfig, ZZZ_DAILY_REWARD,
};
use storekeeper_client_kuro::load_oauth_from_cache;
use storekeeper_core::{AppConfig, DynDailyRewardClient, DynGameClient, Region, SecretsConfig};
use storekeeper_game_genshin::GenshinClient;
use storekeeper_game_hsr::HsrClient;
use storekeeper_game_wuwa::WuwaClient;
use storekeeper_game_zzz::ZzzClient;

use crate::daily_reward_registry::DailyRewardRegistry;
use crate::registry::GameClientRegistry;

type RegionDetector = fn(&str) -> std::result::Result<Region, storekeeper_core::Error>;
type HoyolabGameFactory = fn(HoyolabClient, &str, Region) -> Box<dyn DynGameClient>;

struct EnabledHoyolabGame<'a> {
    uid: &'a str,
    region_override: Option<Region>,
    detect_region: RegionDetector,
    create_client: HoyolabGameFactory,
    game_name: &'static str,
}

struct DailyRewardSpec {
    enabled: bool,
    config: &'static HoyolabDailyRewardConfig,
    game_name: &'static str,
}

fn enabled_hoyolab_games(config: &AppConfig) -> Vec<EnabledHoyolabGame<'_>> {
    let mut games = Vec::new();

    if let Some(c) = config.games.genshin_impact.as_ref().filter(|c| c.enabled) {
        games.push(EnabledHoyolabGame {
            uid: &c.uid,
            region_override: c.region,
            detect_region: Region::from_genshin_uid,
            create_client: |h, uid, region| Box::new(GenshinClient::new(h, uid, region)),
            game_name: "Genshin Impact",
        });
    }

    if let Some(c) = config.games.honkai_star_rail.as_ref().filter(|c| c.enabled) {
        games.push(EnabledHoyolabGame {
            uid: &c.uid,
            region_override: c.region,
            detect_region: Region::from_hsr_uid,
            create_client: |h, uid, region| Box::new(HsrClient::new(h, uid, region)),
            game_name: "Honkai: Star Rail",
        });
    }

    if let Some(c) = config
        .games
        .zenless_zone_zero
        .as_ref()
        .filter(|c| c.enabled)
    {
        games.push(EnabledHoyolabGame {
            uid: &c.uid,
            region_override: c.region,
            detect_region: Region::from_zzz_uid,
            create_client: |h, uid, region| Box::new(ZzzClient::new(h, uid, region)),
            game_name: "Zenless Zone Zero",
        });
    }

    games
}

fn daily_reward_specs(config: &AppConfig) -> [DailyRewardSpec; 3] {
    [
        DailyRewardSpec {
            enabled: config
                .games
                .genshin_impact
                .as_ref()
                .is_some_and(|c| c.enabled),
            config: &GENSHIN_DAILY_REWARD,
            game_name: "Genshin Impact",
        },
        DailyRewardSpec {
            enabled: config
                .games
                .honkai_star_rail
                .as_ref()
                .is_some_and(|c| c.enabled),
            config: &HSR_DAILY_REWARD,
            game_name: "Honkai: Star Rail",
        },
        DailyRewardSpec {
            enabled: config
                .games
                .zenless_zone_zero
                .as_ref()
                .is_some_and(|c| c.enabled),
            config: &ZZZ_DAILY_REWARD,
            game_name: "Zenless Zone Zero",
        },
    ]
}

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

        match HoyolabClient::new(ltuid, ltoken) {
            Ok(hoyolab) => {
                for game in enabled_hoyolab_games(config) {
                    register_hoyolab_game(
                        &mut registry,
                        &hoyolab,
                        game.uid,
                        game.region_override,
                        game.detect_region,
                        game.create_client,
                        game.game_name,
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Failed to create shared HoYoLab client: {e}");
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

    for spec in daily_reward_specs(config)
        .into_iter()
        .filter(|spec| spec.enabled)
    {
        let client = HoyolabDailyRewardClient::new(hoyolab.clone(), spec.config);
        tracing::info!("{} daily reward client registered", spec.game_name);
        registry.register(Box::new(client) as Box<dyn DynDailyRewardClient>);
    }

    tracing::info!(
        client_count = registry.len(),
        "Daily reward registry creation complete"
    );

    registry
}
