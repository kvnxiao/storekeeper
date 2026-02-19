//! Config diff computation for selective reload.
//!
//! Compares old and new configurations to determine the minimum work needed
//! when the user saves settings, avoiding unnecessary HTTP API calls.

use std::collections::HashSet;

use storekeeper_core::{AppConfig, GameId, SecretsConfig};

/// Describes what changed between two configurations.
pub(crate) struct ConfigDiff {
    /// Whether the locale/language setting changed (requires tray rebuild).
    pub locale_changed: bool,

    /// Whether the autostart setting changed (requires OS sync).
    pub autostart_changed: bool,

    /// Whether game client registries need to be rebuilt.
    ///
    /// True when any game's client-relevant fields (enabled, uid, region,
    /// tracked_resources) or provider credentials changed.
    pub needs_registry_rebuild: bool,

    /// Games whose resources should be re-fetched from API.
    pub games_to_refresh: HashSet<GameId>,

    /// Games whose notification cooldowns should be reset.
    pub games_to_reset_notifications: HashSet<GameId>,
}

impl ConfigDiff {
    /// Returns true if nothing changed (no work needed).
    pub fn is_empty(&self) -> bool {
        !self.locale_changed
            && !self.autostart_changed
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
    let locale_changed = old_config.general.language != new_config.general.language;
    let autostart_changed = old_config.general.autostart != new_config.general.autostart;

    let mut needs_registry_rebuild = false;
    let mut games_to_refresh = HashSet::new();
    let mut games_to_reset_notifications = HashSet::new();

    // Check per-game client config changes
    check_game_config(
        GameId::GenshinImpact,
        old_config.games.genshin_impact.as_ref(),
        new_config.games.genshin_impact.as_ref(),
        &mut needs_registry_rebuild,
        &mut games_to_refresh,
        &mut games_to_reset_notifications,
    );
    check_game_config(
        GameId::HonkaiStarRail,
        old_config.games.honkai_star_rail.as_ref(),
        new_config.games.honkai_star_rail.as_ref(),
        &mut needs_registry_rebuild,
        &mut games_to_refresh,
        &mut games_to_reset_notifications,
    );
    check_game_config(
        GameId::ZenlessZoneZero,
        old_config.games.zenless_zone_zero.as_ref(),
        new_config.games.zenless_zone_zero.as_ref(),
        &mut needs_registry_rebuild,
        &mut games_to_refresh,
        &mut games_to_reset_notifications,
    );
    check_game_config(
        GameId::WutheringWaves,
        old_config.games.wuthering_waves.as_ref(),
        new_config.games.wuthering_waves.as_ref(),
        &mut needs_registry_rebuild,
        &mut games_to_refresh,
        &mut games_to_reset_notifications,
    );

    // Check secrets changes — affects all games of the corresponding provider
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
        locale_changed,
        autostart_changed,
        needs_registry_rebuild,
        games_to_refresh,
        games_to_reset_notifications,
    }
}

/// Trait to extract client-relevant fields from any game config for comparison.
///
/// "Client-relevant" means fields that affect the HTTP client setup or what data
/// is fetched. Changes to these fields require a registry rebuild and API re-fetch.
trait ClientFields {
    fn enabled(&self) -> bool;
    fn client_identity(&self) -> ClientIdentity<'_>;
    fn notification_changed(&self, other: &Self) -> bool;
}

/// Identity fields that determine the HTTP client configuration.
#[derive(PartialEq, Eq)]
struct ClientIdentity<'a> {
    enabled: bool,
    uid: &'a str,
    region: Option<&'a storekeeper_core::region::Region>,
    tracked_resources_hash: u64,
}

fn hash_vec<T: std::hash::Hash>(items: &[T]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    items.hash(&mut hasher);
    hasher.finish()
}

impl ClientFields for storekeeper_core::GenshinConfig {
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn client_identity(&self) -> ClientIdentity<'_> {
        ClientIdentity {
            enabled: self.enabled,
            uid: &self.uid,
            region: self.region.as_ref(),
            tracked_resources_hash: hash_vec(&self.tracked_resources),
        }
    }
    fn notification_changed(&self, other: &Self) -> bool {
        self.notifications != other.notifications
    }
}

impl ClientFields for storekeeper_core::HsrConfig {
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn client_identity(&self) -> ClientIdentity<'_> {
        ClientIdentity {
            enabled: self.enabled,
            uid: &self.uid,
            region: self.region.as_ref(),
            tracked_resources_hash: hash_vec(&self.tracked_resources),
        }
    }
    fn notification_changed(&self, other: &Self) -> bool {
        self.notifications != other.notifications
    }
}

impl ClientFields for storekeeper_core::ZzzConfig {
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn client_identity(&self) -> ClientIdentity<'_> {
        ClientIdentity {
            enabled: self.enabled,
            uid: &self.uid,
            region: self.region.as_ref(),
            tracked_resources_hash: hash_vec(&self.tracked_resources),
        }
    }
    fn notification_changed(&self, other: &Self) -> bool {
        self.notifications != other.notifications
    }
}

impl ClientFields for storekeeper_core::WuwaConfig {
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn client_identity(&self) -> ClientIdentity<'_> {
        ClientIdentity {
            enabled: self.enabled,
            uid: &self.player_id,
            region: self.region.as_ref(),
            tracked_resources_hash: hash_vec(&self.tracked_resources),
        }
    }
    fn notification_changed(&self, other: &Self) -> bool {
        self.notifications != other.notifications
    }
}

/// Compares a single game's config and updates the diff accumulators.
fn check_game_config<T: ClientFields>(
    game_id: GameId,
    old: Option<&T>,
    new: Option<&T>,
    needs_registry_rebuild: &mut bool,
    games_to_refresh: &mut HashSet<GameId>,
    games_to_reset_notifications: &mut HashSet<GameId>,
) {
    match (old, new) {
        (None, None) => {}
        // Game added or removed — needs rebuild + refresh
        (None, Some(cfg)) if cfg.enabled() => {
            *needs_registry_rebuild = true;
            games_to_refresh.insert(game_id);
        }
        (Some(cfg), None) if cfg.enabled() => {
            *needs_registry_rebuild = true;
        }
        (None, Some(_)) | (Some(_), None) => {
            // Disabled game added/removed — rebuild but no fetch needed
            *needs_registry_rebuild = true;
        }
        (Some(old_cfg), Some(new_cfg)) => {
            // Check client-relevant fields
            if old_cfg.client_identity() != new_cfg.client_identity() {
                *needs_registry_rebuild = true;
                if new_cfg.enabled() {
                    games_to_refresh.insert(game_id);
                }
            }

            // Check notification config changes
            if old_cfg.notification_changed(new_cfg) {
                games_to_reset_notifications.insert(game_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use storekeeper_core::config::secrets::{HoyolabSecrets, KuroSecrets};
    use storekeeper_core::{AppConfig, GamesConfig, GenshinConfig, SecretsConfig, WuwaConfig};

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
            player_id: "987654321".to_string(),
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

        assert!(diff.locale_changed);
        assert!(!diff.autostart_changed);
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

        assert!(!diff.locale_changed);
        assert!(diff.autostart_changed);
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
        // Disabled game doesn't need refresh
        assert!(!diff.games_to_refresh.contains(&GameId::GenshinImpact));
    }

    #[test]
    fn notification_change_only_resets_cooldowns() {
        let old = config_with_genshin(default_genshin());
        let mut new_genshin = default_genshin();
        new_genshin.notifications.insert(
            storekeeper_core::resource_types::GenshinResourceType::Resin,
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
        // Only enabled HoYoLab games get refreshed
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
