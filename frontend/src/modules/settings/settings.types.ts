/** Match the configuration shapes serialized by the Rust backend. */

import type { ResourceType } from "@/modules/games/games.constants";

/** General application settings */
export interface GeneralConfig {
  poll_interval_secs: number;
  start_minimized: boolean;
  /** Accept a `tracing_subscriber::EnvFilter` directive, not only a log level. */
  log_level: string;
  /** Store an optional locale override; the backend omits it when unset. */
  language?: string | null;
  autostart: boolean;
}

/** Per-resource notification configuration */
export interface ResourceNotificationConfig {
  enabled: boolean;
  notify_minutes_before_full?: number | null;
  notify_at_value?: number | null;
  cooldown_minutes: number;
}

/** Common HoYoLab configuration; `GAME_REGISTRY` supplies each game's resource set. */
export interface HoyolabGameConfig {
  enabled: boolean;
  uid: string;
  tracked_resources: ResourceType[];
  auto_claim_daily_rewards: boolean;
  /** Use "HH:MM" in UTC+8. */
  auto_claim_time: string | null;
  notifications: Partial<Record<ResourceType, ResourceNotificationConfig>>;
}

/** Wuthering Waves configuration */
export interface WuwaConfig {
  enabled: boolean;
  uid: string;
  tracked_resources: ResourceType[];
  notifications: Partial<Record<ResourceType, ResourceNotificationConfig>>;
}

/** Per-game configuration */
export interface GamesConfig {
  genshin_impact: HoyolabGameConfig | null;
  honkai_star_rail: HoyolabGameConfig | null;
  zenless_zone_zero: HoyolabGameConfig | null;
  wuthering_waves: WuwaConfig | null;
}

/** Config keys for HoYoLab games only */
export type HoyolabConfigKey = Exclude<keyof GamesConfig, "wuthering_waves">;

/** Main application configuration (config.toml) */
export interface AppConfig {
  general: GeneralConfig;
  games: GamesConfig;
}

/** HoYoLab authentication secrets */
export interface HoyolabSecrets {
  ltuid_v2: string;
  ltoken_v2: string;
  ltmid_v2: string;
}

/** Kuro Games authentication secrets */
export interface KuroSecrets {
  oauth_code: string;
}

/** Secrets configuration (secrets.toml) */
export interface SecretsConfig {
  hoyolab: HoyolabSecrets;
  kuro: KuroSecrets;
}

/** Result returned by the save_and_apply command */
export interface SaveResult {
  effective_locale: string;
}
