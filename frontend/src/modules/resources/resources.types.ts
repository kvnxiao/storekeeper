import type { GameId } from "@/modules/games/games.types";

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

/** Payload for per-game resource update events */
export interface GameResourcePayload {
  gameId: GameId;
  data: GameResource[];
}

/** All resources from all games (camelCase from Rust) */
export interface AllResources {
  games?: Partial<Record<GameId, GameResource[]>>;
  lastUpdated?: string; // ISO 8601 datetime
}

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
