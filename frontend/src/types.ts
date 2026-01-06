/** Game ID enum matching Rust GameId */
export type GameId =
  | "genshin_impact"
  | "honkai_star_rail"
  | "zenless_zone_zero"
  | "wuthering_waves";

/** Stamina resource data (camelCase from Rust) */
export interface StaminaResource {
  current: number;
  max: number;
  fullAt: string; // ISO 8601 datetime
  regenRateSeconds: number;
}

/** Cooldown resource data (camelCase from Rust) */
export interface CooldownResource {
  isReady: boolean;
  readyAt: string; // ISO 8601 datetime
}

/** Expedition resource data (camelCase from Rust) */
export interface ExpeditionResource {
  currentExpeditions: number;
  maxExpeditions: number;
  earliestFinishAt: string; // ISO 8601 datetime
}

/** Game resource with tagged type and data */
export interface GameResource {
  type: string;
  data: StaminaResource | CooldownResource | ExpeditionResource;
}

/** All resources from all games (camelCase from Rust) */
export interface AllResources {
  games: Record<GameId, GameResource[]>;
  lastUpdated?: string; // ISO 8601 datetime
}

/** Game metadata for display */
export interface GameMetadata {
  title: string;
  shortId: string;
}

/** Game metadata lookup */
export const GAME_METADATA: Record<GameId, GameMetadata> = {
  genshin_impact: { title: "Genshin Impact", shortId: "genshin" },
  honkai_star_rail: { title: "Honkai: Star Rail", shortId: "hsr" },
  zenless_zone_zero: { title: "Zenless Zone Zero", shortId: "zzz" },
  wuthering_waves: { title: "Wuthering Waves", shortId: "wuwa" },
};

/** Ordered list of games */
export const GAME_ORDER: GameId[] = [
  "genshin_impact",
  "honkai_star_rail",
  "zenless_zone_zero",
  "wuthering_waves",
];

/** Type guards */
export function isStaminaResource(data: unknown): data is StaminaResource {
  return (
    typeof data === "object" &&
    data !== null &&
    "current" in data &&
    "max" in data
  );
}

export function isCooldownResource(data: unknown): data is CooldownResource {
  return typeof data === "object" && data !== null && "isReady" in data;
}

export function isExpeditionResource(
  data: unknown,
): data is ExpeditionResource {
  return (
    typeof data === "object" && data !== null && "currentExpeditions" in data
  );
}
