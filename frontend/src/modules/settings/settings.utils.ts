import { GAMES } from "@/modules/games/games.registry";
import type { GameId } from "@/modules/games/games.types";
import type { AppConfig, ResourceNotificationConfig } from "@/modules/settings/settings.types";

/** Returns the set of games enabled in config; empty while config is unloaded. */
export function enabledGamesFromConfig(config: AppConfig | undefined): Set<GameId> {
  return new Set(
    GAMES.filter((game) => config?.games[game.configKey]?.enabled).map((game) => game.gameId),
  );
}

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
 * Returns the enabled config for a resource, clearing both thresholds for
 * cooldown resources so the backend notifies on completion instead.
 */
export function enabledNotificationConfig(
  config: ResourceNotificationConfig | undefined,
  isStaminaResource: boolean,
): ResourceNotificationConfig {
  if (!config) {
    return { ...DEFAULT_NOTIFICATION_CONFIG };
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
  // The value-mode field's minimum is 1; a 0 default would persist a
  // threshold the UI refuses to let the user enter.
  return mode === "value"
    ? { ...config, notify_minutes_before_full: null, notify_at_value: config.notify_at_value ?? 1 }
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
