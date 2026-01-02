//! Game client initialization for the registry.

use storekeeper_client_kuro::load_oauth_from_cache;
use storekeeper_core::{AppConfig, DynDailyRewardClient, DynGameClient, Region, SecretsConfig};
use storekeeper_game_genshin::GenshinClient;
use storekeeper_game_hsr::HsrClient;
use storekeeper_game_wuwa::WuwaClient;
use storekeeper_game_zzz::ZzzClient;

use crate::daily_reward_registry::DailyRewardRegistry;
use crate::registry::GameClientRegistry;

/// Creates a `GameClientRegistry` from configuration and secrets.
///
/// This function creates type-erased game clients that can be stored
/// in a single registry for dynamic dispatch.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn create_registry(config: &AppConfig, secrets: &SecretsConfig) -> GameClientRegistry {
    tracing::info!("Creating game client registry from configuration");
    let mut registry = GameClientRegistry::new();

    // Initialize HoYoLab-based clients if credentials are configured
    if secrets.hoyolab.is_configured() {
        tracing::debug!("HoYoLab credentials found, initializing HoYoLab-based clients");
        let ltuid = secrets.hoyolab.ltuid();
        let ltoken = secrets.hoyolab.ltoken();

        // Genshin Impact
        if let Some(ref genshin_config) = config.games.genshin_impact {
            if genshin_config.enabled {
                let region = genshin_config
                    .region
                    .or_else(|| Region::from_genshin_uid(&genshin_config.uid).ok());
                if let Some(region) = region {
                    if let Ok(client) =
                        GenshinClient::new(ltuid, ltoken, &genshin_config.uid, region)
                    {
                        tracing::info!(
                            uid = %genshin_config.uid,
                            region = ?region,
                            "Genshin Impact client registered"
                        );
                        registry.register(Box::new(client) as Box<dyn DynGameClient>);
                    }
                }
            }
        }

        // Honkai: Star Rail
        if let Some(ref hsr_config) = config.games.honkai_star_rail {
            if hsr_config.enabled {
                let region = hsr_config
                    .region
                    .or_else(|| Region::from_hsr_uid(&hsr_config.uid).ok());
                if let Some(region) = region {
                    if let Ok(client) = HsrClient::new(ltuid, ltoken, &hsr_config.uid, region) {
                        tracing::info!(
                            uid = %hsr_config.uid,
                            region = ?region,
                            "Honkai: Star Rail client registered"
                        );
                        registry.register(Box::new(client) as Box<dyn DynGameClient>);
                    }
                }
            }
        }

        // Zenless Zone Zero
        if let Some(ref zzz_config) = config.games.zenless_zone_zero {
            if zzz_config.enabled {
                let region = zzz_config
                    .region
                    .or_else(|| Region::from_zzz_uid(&zzz_config.uid).ok());
                if let Some(region) = region {
                    if let Ok(client) = ZzzClient::new(ltuid, ltoken, &zzz_config.uid, region) {
                        tracing::info!(
                            uid = %zzz_config.uid,
                            region = ?region,
                            "Zenless Zone Zero client registered"
                        );
                        registry.register(Box::new(client) as Box<dyn DynGameClient>);
                    }
                }
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
/// This function creates type-erased daily reward clients that can be stored
/// in a single registry for dynamic dispatch. Only HoYoLab games are supported
/// for daily rewards.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn create_daily_reward_registry(
    config: &AppConfig,
    secrets: &SecretsConfig,
) -> DailyRewardRegistry {
    tracing::info!("Creating daily reward registry from configuration");
    let mut registry = DailyRewardRegistry::new();

    // Only HoYoLab games support daily rewards
    if secrets.hoyolab.is_configured() {
        tracing::debug!("HoYoLab credentials found, initializing daily reward clients");
        let ltuid = secrets.hoyolab.ltuid();
        let ltoken = secrets.hoyolab.ltoken();

        // Genshin Impact
        if let Some(ref genshin_config) = config.games.genshin_impact {
            if genshin_config.enabled {
                let region = genshin_config
                    .region
                    .or_else(|| Region::from_genshin_uid(&genshin_config.uid).ok());
                if let Some(region) = region {
                    if let Ok(client) =
                        GenshinClient::new(ltuid, ltoken, &genshin_config.uid, region)
                    {
                        tracing::info!(
                            uid = %genshin_config.uid,
                            region = ?region,
                            "Genshin Impact daily reward client registered"
                        );
                        registry.register(Box::new(client) as Box<dyn DynDailyRewardClient>);
                    }
                }
            }
        }

        // Honkai: Star Rail
        if let Some(ref hsr_config) = config.games.honkai_star_rail {
            if hsr_config.enabled {
                let region = hsr_config
                    .region
                    .or_else(|| Region::from_hsr_uid(&hsr_config.uid).ok());
                if let Some(region) = region {
                    if let Ok(client) = HsrClient::new(ltuid, ltoken, &hsr_config.uid, region) {
                        tracing::info!(
                            uid = %hsr_config.uid,
                            region = ?region,
                            "Honkai: Star Rail daily reward client registered"
                        );
                        registry.register(Box::new(client) as Box<dyn DynDailyRewardClient>);
                    }
                }
            }
        }

        // Zenless Zone Zero
        if let Some(ref zzz_config) = config.games.zenless_zone_zero {
            if zzz_config.enabled {
                let region = zzz_config
                    .region
                    .or_else(|| Region::from_zzz_uid(&zzz_config.uid).ok());
                if let Some(region) = region {
                    if let Ok(client) = ZzzClient::new(ltuid, ltoken, &zzz_config.uid, region) {
                        tracing::info!(
                            uid = %zzz_config.uid,
                            region = ?region,
                            "Zenless Zone Zero daily reward client registered"
                        );
                        registry.register(Box::new(client) as Box<dyn DynDailyRewardClient>);
                    }
                }
            }
        }
    } else {
        tracing::debug!("HoYoLab credentials not configured, skipping daily reward clients");
    }

    tracing::info!(
        client_count = registry.len(),
        "Daily reward registry creation complete"
    );

    registry
}
