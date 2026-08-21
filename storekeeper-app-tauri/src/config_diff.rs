//! Config diff computation for selective reload.
//!
//! Compares old and new configurations to determine the minimum work needed
//! when the user saves settings, avoiding unnecessary HTTP API calls.

use std::collections::HashSet;
use storekeeper_core::AppConfig;
use storekeeper_core::GameId;
use storekeeper_core::SecretsConfig;

/// Track general settings that `save_and_apply` applies independently.
pub(crate) struct GeneralDiff {
    /// Whether the locale or language changed and requires a tray rebuild.
    pub locale_changed: bool,

    /// Whether autostart changed and requires OS synchronization.
    pub autostart_changed: bool,

    /// Whether the log level changed and requires a filter swap.
    pub log_level_changed: bool,
}

impl GeneralDiff {
    fn is_empty(&self) -> bool {
        !self.locale_changed && !self.autostart_changed && !self.log_level_changed
    }
}

/// Track the configuration changes that determine selective application work.
pub(crate) struct ConfigDiff {
    pub general: GeneralDiff,

    /// Whether game client registries need to be rebuilt.
    ///
    /// True when any game's client-relevant fields (enabled, uid, region,
    /// `tracked_resources`) or provider credentials changed.
    pub needs_registry_rebuild: bool,

    /// Games whose resources should be re-fetched from API.
    pub games_to_refresh: HashSet<GameId>,

    /// Games whose notification cooldowns should be reset.
    pub games_to_reset_notifications: HashSet<GameId>,
}

impl ConfigDiff {
    /// Returns true if nothing changed (no work needed).
    pub fn is_empty(&self) -> bool {
        self.general.is_empty()
            && !self.needs_registry_rebuild
            && self.games_to_refresh.is_empty()
            && self.games_to_reset_notifications.is_empty()
    }
}

/// Computes the diff between old and new config/secrets.
pub(crate) fn compute(
    old_config: &AppConfig,
    new_config: &AppConfig,
    old_secrets: &SecretsConfig,
    new_secrets: &SecretsConfig,
) -> ConfigDiff {
    let general = GeneralDiff {
        locale_changed: old_config.general.language != new_config.general.language,
        autostart_changed: old_config.general.autostart != new_config.general.autostart,
        log_level_changed: old_config.general.log_level != new_config.general.log_level,
    };

    let mut needs_registry_rebuild = false;
    let mut games_to_refresh = HashSet::new();
    let mut games_to_reset_notifications = HashSet::new();

    for change in [
        check_game_config(
            GameId::GenshinImpact,
            old_config.games.genshin_impact.as_ref(),
            new_config.games.genshin_impact.as_ref(),
        ),
        check_game_config(
            GameId::HonkaiStarRail,
            old_config.games.honkai_star_rail.as_ref(),
            new_config.games.honkai_star_rail.as_ref(),
        ),
        check_game_config(
            GameId::ZenlessZoneZero,
            old_config.games.zenless_zone_zero.as_ref(),
            new_config.games.zenless_zone_zero.as_ref(),
        ),
        check_game_config(
            GameId::WutheringWaves,
            old_config.games.wuthering_waves.as_ref(),
            new_config.games.wuthering_waves.as_ref(),
        ),
    ] {
        needs_registry_rebuild |= change.needs_registry_rebuild;
        games_to_refresh.extend(change.game_to_refresh);
        games_to_reset_notifications.extend(change.game_to_reset_notifications);
    }

    // A secrets change invalidates every game under that provider.
    if old_secrets.hoyolab != new_secrets.hoyolab {
        needs_registry_rebuild = true;
        for &game_id in &[
            GameId::GenshinImpact,
            GameId::HonkaiStarRail,
            GameId::ZenlessZoneZero,
        ] {
            if new_config.games.is_enabled(game_id) {
                games_to_refresh.insert(game_id);
            }
        }
    }

    if old_secrets.kuro != new_secrets.kuro {
        needs_registry_rebuild = true;
        if new_config.games.is_enabled(GameId::WutheringWaves) {
            games_to_refresh.insert(GameId::WutheringWaves);
        }
    }

    ConfigDiff {
        general,
        needs_registry_rebuild,
        games_to_refresh,
        games_to_reset_notifications,
    }
}

/// Trait to extract client-relevant fields from any game config for comparison.
///
/// "Client-relevant" means fields that affect the HTTP client setup or what
/// data is fetched. Changes to these fields require a registry rebuild and API
/// re-fetch.
trait ClientFields {
    /// The game's resource enum, compared field-wise between two configs.
    type Resource: Eq;

    fn enabled(&self) -> bool;
    fn client_identity(&self) -> ClientIdentity<'_, Self::Resource>;
    fn notification_changed(&self, other: &Self) -> bool;
}

/// Identity fields that determine the HTTP client configuration.
#[derive(PartialEq, Eq)]
struct ClientIdentity<'a, R> {
    enabled: bool,
    uid: &'a str,
    region: Option<&'a storekeeper_core::Region>,
    tracked_resources: &'a [R],
}

impl ClientFields for storekeeper_core::GenshinConfig {
    type Resource = storekeeper_core::GenshinResourceType;

    fn enabled(&self) -> bool {
        self.enabled
    }
    fn client_identity(&self) -> ClientIdentity<'_, Self::Resource> {
        ClientIdentity {
            enabled: self.enabled,
            uid: &self.uid,
            region: self.region.as_ref(),
            tracked_resources: &self.tracked_resources,
        }
    }
    fn notification_changed(&self, other: &Self) -> bool {
        self.notifications != other.notifications
    }
}

impl ClientFields for storekeeper_core::HsrConfig {
    type Resource = storekeeper_core::HsrResourceType;

    fn enabled(&self) -> bool {
        self.enabled
    }
    fn client_identity(&self) -> ClientIdentity<'_, Self::Resource> {
        ClientIdentity {
            enabled: self.enabled,
            uid: &self.uid,
            region: self.region.as_ref(),
            tracked_resources: &self.tracked_resources,
        }
    }
    fn notification_changed(&self, other: &Self) -> bool {
        self.notifications != other.notifications
    }
}

impl ClientFields for storekeeper_core::ZzzConfig {
    type Resource = storekeeper_core::ZzzResourceType;

    fn enabled(&self) -> bool {
        self.enabled
    }
    fn client_identity(&self) -> ClientIdentity<'_, Self::Resource> {
        ClientIdentity {
            enabled: self.enabled,
            uid: &self.uid,
            region: self.region.as_ref(),
            tracked_resources: &self.tracked_resources,
        }
    }
    fn notification_changed(&self, other: &Self) -> bool {
        self.notifications != other.notifications
    }
}

impl ClientFields for storekeeper_core::WuwaConfig {
    type Resource = storekeeper_core::WuwaResourceType;

    fn enabled(&self) -> bool {
        self.enabled
    }
    fn client_identity(&self) -> ClientIdentity<'_, Self::Resource> {
        ClientIdentity {
            enabled: self.enabled,
            uid: &self.uid,
            region: self.region.as_ref(),
            tracked_resources: &self.tracked_resources,
        }
    }
    fn notification_changed(&self, other: &Self) -> bool {
        self.notifications != other.notifications
    }
}

/// What changed for a single game's config.
struct GameConfigChange {
    needs_registry_rebuild: bool,
    game_to_refresh: Option<GameId>,
    game_to_reset_notifications: Option<GameId>,
}

/// Compares a single game's config and returns what changed.
fn check_game_config<T: ClientFields>(
    game_id: GameId,
    old: Option<&T>,
    new: Option<&T>,
) -> GameConfigChange {
    let none = GameConfigChange {
        needs_registry_rebuild: false,
        game_to_refresh: None,
        game_to_reset_notifications: None,
    };

    match (old, new) {
        (None, None) => none,
        (None, Some(cfg)) if cfg.enabled() => GameConfigChange {
            needs_registry_rebuild: true,
            game_to_refresh: Some(game_id),
            ..none
        },
        (Some(cfg), None) if cfg.enabled() => GameConfigChange {
            needs_registry_rebuild: true,
            ..none
        },
        (None, Some(_)) | (Some(_), None) => GameConfigChange {
            needs_registry_rebuild: true,
            ..none
        },
        (Some(old_cfg), Some(new_cfg)) => {
            let identity_changed = old_cfg.client_identity() != new_cfg.client_identity();
            GameConfigChange {
                needs_registry_rebuild: identity_changed,
                game_to_refresh: if identity_changed && new_cfg.enabled() {
                    Some(game_id)
                } else {
                    None
                },
                game_to_reset_notifications: if old_cfg.notification_changed(new_cfg) {
                    Some(game_id)
                } else {
                    None
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use storekeeper_core::AppConfig;
    use storekeeper_core::GamesConfig;
    use storekeeper_core::GenshinConfig;
    use storekeeper_core::HoyolabSecrets;
    use storekeeper_core::KuroSecrets;
    use storekeeper_core::SecretsConfig;
    use storekeeper_core::WuwaConfig;

    fn default_genshin() -> GenshinConfig {
        GenshinConfig {
            enabled: true,
            uid: "123456789".to_string(),
            region: None,
            tracked_resources: storekeeper_core::GenshinResourceType::all().to_vec(),
            auto_claim_daily_rewards: false,
            auto_claim_time: None,
            notifications: std::collections::HashMap::new(),
        }
    }

    fn default_wuwa() -> WuwaConfig {
        WuwaConfig {
            enabled: true,
            uid: "987654321".to_string(),
            region: None,
            tracked_resources: storekeeper_core::WuwaResourceType::all().to_vec(),
            notifications: std::collections::HashMap::new(),
        }
    }

    fn config_with_genshin(genshin: GenshinConfig) -> AppConfig {
        AppConfig {
            games: GamesConfig {
                genshin_impact: Some(genshin),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn config_with_wuwa(wuwa: WuwaConfig) -> AppConfig {
        AppConfig {
            games: GamesConfig {
                wuthering_waves: Some(wuwa),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn no_changes_produces_empty_diff() {
        let config = AppConfig::default();
        let secrets = SecretsConfig::default();
        let diff = compute(&config, &config, &secrets, &secrets);
        assert!(diff.is_empty());
    }

    #[test]
    fn language_change_only() {
        let mut old = AppConfig::default();
        old.general.language = Some("en".to_string());
        let mut new = old.clone();
        new.general.language = Some("zh-CN".to_string());

        let secrets = SecretsConfig::default();
        let diff = compute(&old, &new, &secrets, &secrets);

        assert!(diff.general.locale_changed);
        assert!(!diff.general.autostart_changed);
        assert!(!diff.needs_registry_rebuild);
        assert!(diff.games_to_refresh.is_empty());
    }

    #[test]
    fn autostart_change_only() {
        let old = AppConfig::default();
        let mut new = old.clone();
        new.general.autostart = true;

        let secrets = SecretsConfig::default();
        let diff = compute(&old, &new, &secrets, &secrets);

        assert!(!diff.general.locale_changed);
        assert!(diff.general.autostart_changed);
        assert!(!diff.needs_registry_rebuild);
        assert!(diff.games_to_refresh.is_empty());
    }

    #[test]
    fn log_level_change_only() {
        let old = AppConfig::default();
        let mut new = old.clone();
        new.general.log_level = "debug".to_string();

        let secrets = SecretsConfig::default();
        let diff = compute(&old, &new, &secrets, &secrets);

        assert!(diff.general.log_level_changed);
        assert!(!diff.is_empty(), "a level change must reach the apply step");
        assert!(!diff.needs_registry_rebuild);
        assert!(diff.games_to_refresh.is_empty());
    }

    #[test]
    fn game_uid_change_triggers_rebuild_and_refresh() {
        let old = config_with_genshin(default_genshin());
        let mut new_genshin = default_genshin();
        new_genshin.uid = "999999999".to_string();
        let new = config_with_genshin(new_genshin);

        let secrets = SecretsConfig::default();
        let diff = compute(&old, &new, &secrets, &secrets);

        assert!(diff.needs_registry_rebuild);
        assert!(diff.games_to_refresh.contains(&GameId::GenshinImpact));
    }

    #[test]
    fn game_enabled_toggle_triggers_rebuild() {
        let old = config_with_genshin(default_genshin());
        let mut new_genshin = default_genshin();
        new_genshin.enabled = false;
        let new = config_with_genshin(new_genshin);

        let secrets = SecretsConfig::default();
        let diff = compute(&old, &new, &secrets, &secrets);

        assert!(diff.needs_registry_rebuild);
        assert!(!diff.games_to_refresh.contains(&GameId::GenshinImpact));
    }

    #[test]
    fn notification_change_only_resets_cooldowns() {
        let old = config_with_genshin(default_genshin());
        let mut new_genshin = default_genshin();
        new_genshin.notifications.insert(
            storekeeper_core::GenshinResourceType::Resin,
            storekeeper_core::ResourceNotificationConfig {
                enabled: true,
                notify_minutes_before_full: Some(30),
                notify_at_value: None,
                cooldown_minutes: 10,
            },
        );
        let new = config_with_genshin(new_genshin);

        let secrets = SecretsConfig::default();
        let diff = compute(&old, &new, &secrets, &secrets);

        assert!(!diff.needs_registry_rebuild);
        assert!(diff.games_to_refresh.is_empty());
        assert!(
            diff.games_to_reset_notifications
                .contains(&GameId::GenshinImpact)
        );
    }

    #[test]
    fn hoyolab_secrets_change_refreshes_all_hoyolab_games() {
        let genshin = default_genshin();
        let old = config_with_genshin(genshin.clone());
        let new = old.clone();

        let old_secrets = SecretsConfig::default();
        let new_secrets = SecretsConfig {
            hoyolab: HoyolabSecrets {
                ltuid_v2: "new_uid".to_string(),
                ltoken_v2: "new_token".to_string(),
                ltmid_v2: "new_mid".to_string(),
            },
            ..Default::default()
        };

        let diff = compute(&old, &new, &old_secrets, &new_secrets);

        assert!(diff.needs_registry_rebuild);
        assert!(diff.games_to_refresh.contains(&GameId::GenshinImpact));
        assert!(!diff.games_to_refresh.contains(&GameId::HonkaiStarRail));
    }

    #[test]
    fn kuro_secrets_change_refreshes_wuwa() {
        let old = config_with_wuwa(default_wuwa());
        let new = old.clone();

        let old_secrets = SecretsConfig::default();
        let new_secrets = SecretsConfig {
            kuro: KuroSecrets {
                oauth_code: "new_code".to_string(),
            },
            ..Default::default()
        };

        let diff = compute(&old, &new, &old_secrets, &new_secrets);

        assert!(diff.needs_registry_rebuild);
        assert!(diff.games_to_refresh.contains(&GameId::WutheringWaves));
    }

    #[test]
    fn game_added_triggers_rebuild_and_refresh() {
        let old = AppConfig::default();
        let new = config_with_genshin(default_genshin());

        let secrets = SecretsConfig::default();
        let diff = compute(&old, &new, &secrets, &secrets);

        assert!(diff.needs_registry_rebuild);
        assert!(diff.games_to_refresh.contains(&GameId::GenshinImpact));
    }

    #[test]
    fn game_removed_triggers_rebuild_no_refresh() {
        let old = config_with_genshin(default_genshin());
        let new = AppConfig::default();

        let secrets = SecretsConfig::default();
        let diff = compute(&old, &new, &secrets, &secrets);

        assert!(diff.needs_registry_rebuild);
        assert!(diff.games_to_refresh.is_empty());
    }

    #[test]
    fn unchanged_config_with_games_is_empty() {
        let config = config_with_genshin(default_genshin());
        let secrets = SecretsConfig::default();
        let diff = compute(&config, &config, &secrets, &secrets);
        assert!(diff.is_empty());
    }
}
