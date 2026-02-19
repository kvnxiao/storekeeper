import type {
  GenshinResourceType,
  HsrResourceType,
  WuwaResourceType,
  ZzzResourceType,
} from "@/modules/games/games.constants";

// =============================================================================
// Configuration Types (matching Rust AppConfig - snake_case)
// =============================================================================

/** General application settings */
export interface GeneralConfig {
  poll_interval_secs: number;
  start_minimized: boolean;
  log_level: string;
  language: string | null;
  autostart: boolean;
}

/** Per-resource notification configuration */
export interface ResourceNotificationConfig {
  enabled: boolean;
  notify_minutes_before_full?: number | null;
  notify_at_value?: number | null;
  cooldown_minutes: number;
}

/** Common configuration for HoYoLab games */
export interface HoyolabGameConfig {
  enabled: boolean;
  uid: string;
  region?: string;
  tracked_resources?: string[];
  auto_claim_daily_rewards: boolean;
  auto_claim_time?: string;
  notifications?: Partial<Record<string, ResourceNotificationConfig>>;
}

/** Genshin Impact configuration */
export interface GenshinConfig extends HoyolabGameConfig {
  notifications?: Partial<
    Record<GenshinResourceType, ResourceNotificationConfig>
  >;
}

/** Honkai: Star Rail configuration */
export interface HsrConfig extends HoyolabGameConfig {
  notifications?: Partial<Record<HsrResourceType, ResourceNotificationConfig>>;
}

/** Zenless Zone Zero configuration */
export interface ZzzConfig extends HoyolabGameConfig {
  notifications?: Partial<Record<ZzzResourceType, ResourceNotificationConfig>>;
}

/** Wuthering Waves configuration */
export interface WuwaConfig {
  enabled: boolean;
  player_id: string;
  region?: string;
  tracked_resources?: string[];
  notifications?: Partial<Record<WuwaResourceType, ResourceNotificationConfig>>;
}

/** Per-game configuration */
export interface GamesConfig {
  genshin_impact?: GenshinConfig;
  honkai_star_rail?: HsrConfig;
  zenless_zone_zero?: ZzzConfig;
  wuthering_waves?: WuwaConfig;
}

/** Config keys for HoYoLab games only */
export type HoyolabConfigKey = Exclude<keyof GamesConfig, "wuthering_waves">;

/** Main application configuration (config.toml) */
export interface AppConfig {
  general: GeneralConfig;
  games: GamesConfig;
}

// =============================================================================
// Secrets Types (matching Rust SecretsConfig - snake_case)
// =============================================================================

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

// =============================================================================
// Command Result Types
// =============================================================================

/** Result returned by the save_and_apply command */
export interface SaveResult {
  effective_locale: string;
}
