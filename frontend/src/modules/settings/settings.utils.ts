import { GameId } from "@/modules/games/games.types";
import type {
  AppConfig,
  GamesConfig,
  ResourceNotificationConfig,
} from "@/modules/settings/settings.types";

const GAME_CONFIG_KEYS: [GameId, keyof GamesConfig][] = [
  [GameId.GenshinImpact, "genshin_impact"],
  [GameId.HonkaiStarRail, "honkai_star_rail"],
  [GameId.ZenlessZoneZero, "zenless_zone_zero"],
  [GameId.WutheringWaves, "wuthering_waves"],
];

/** Returns the set of games enabled in config; empty while config is unloaded. */
export function enabledGamesFromConfig(config: AppConfig | undefined): Set<GameId> {
  return new Set<GameId>(
    GAME_CONFIG_KEYS.filter(([, key]) => config?.games[key]?.enabled).map(([id]) => id),
  );
}

// =============================================================================
// Resource notification contract (mirrors backend threshold semantics)
// =============================================================================

export type NotifyMode = "minutes" | "value";

export const DEFAULT_NOTIFICATION_CONFIG: ResourceNotificationConfig = {
  enabled: true,
  cooldown_minutes: 30,
};

/** Returns which threshold mode a notification config is in. */
export function getNotifyMode(config: ResourceNotificationConfig): NotifyMode {
  return config.notify_at_value != null ? "value" : "minutes";
}

/**
 * Returns the enabled config for a resource. Cooldown resources must carry
 * null threshold fields so the backend notifies on completion.
 */
export function enabledNotificationConfig(
  config: ResourceNotificationConfig | undefined,
  isStaminaResource: boolean,
): ResourceNotificationConfig {
  if (!config) {
    return DEFAULT_NOTIFICATION_CONFIG;
  }
  if (isStaminaResource) {
    return { ...config, enabled: true };
  }
  return { ...config, enabled: true, notify_minutes_before_full: null, notify_at_value: null };
}

/** Switches threshold mode; the two threshold fields are mutually exclusive. */
export function withNotifyMode(
  config: ResourceNotificationConfig,
  mode: NotifyMode,
): ResourceNotificationConfig {
  return mode === "value"
    ? { ...config, notify_minutes_before_full: null, notify_at_value: config.notify_at_value ?? 0 }
    : {
        ...config,
        notify_at_value: null,
        notify_minutes_before_full: config.notify_minutes_before_full ?? 0,
      };
}

/** Sets the notify-at-value threshold, clearing the minutes threshold. */
export function withNotifyAtValue(
  config: ResourceNotificationConfig,
  value: number,
): ResourceNotificationConfig {
  return { ...config, notify_at_value: value, notify_minutes_before_full: null };
}

/** Sets the minutes-before-full threshold, clearing the value threshold. */
export function withNotifyMinutes(
  config: ResourceNotificationConfig,
  value: number,
): ResourceNotificationConfig {
  return { ...config, notify_minutes_before_full: value, notify_at_value: null };
}
