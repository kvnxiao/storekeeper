// =============================================================================
// Configuration Types (matching Rust AppConfig - snake_case)
// =============================================================================

/** General application settings */
export interface GeneralConfig {
  poll_interval_secs: number;
  start_minimized: boolean;
  log_level: string;
}

/** Notification settings */
export interface NotificationConfig {
  enabled: boolean;
  cooldown_minutes: number;
  thresholds: Record<string, ThresholdConfig>;
}

/** Threshold configuration for a specific resource */
export interface ThresholdConfig {
  enabled: boolean;
  threshold_value: number;
  notify_when_full: boolean;
}

/** Genshin Impact configuration */
export interface GenshinConfig {
  enabled: boolean;
  uid: string;
  region?: string;
  tracked_resources?: string[];
  auto_claim_daily_rewards: boolean;
  auto_claim_time?: string;
}

/** Honkai: Star Rail configuration */
export interface HsrConfig {
  enabled: boolean;
  uid: string;
  region?: string;
  tracked_resources?: string[];
  auto_claim_daily_rewards: boolean;
  auto_claim_time?: string;
}

/** Zenless Zone Zero configuration */
export interface ZzzConfig {
  enabled: boolean;
  uid: string;
  region?: string;
  tracked_resources?: string[];
  auto_claim_daily_rewards: boolean;
  auto_claim_time?: string;
}

/** Wuthering Waves configuration */
export interface WuwaConfig {
  enabled: boolean;
  player_id: string;
  region?: string;
  tracked_resources?: string[];
}

/** Per-game configuration */
export interface GamesConfig {
  genshin_impact?: GenshinConfig;
  honkai_star_rail?: HsrConfig;
  zenless_zone_zero?: ZzzConfig;
  wuthering_waves?: WuwaConfig;
}

/** Main application configuration (config.toml) */
export interface AppConfig {
  general: GeneralConfig;
  notifications: NotificationConfig;
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
