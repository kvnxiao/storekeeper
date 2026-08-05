import type { GameId } from "@/modules/games/games.types";

/** Backend response shape for daily-reward status. */
export interface AllDailyRewardStatus {
  games?: Record<string, { info?: { is_signed?: boolean } }>;
  lastChecked?: string;
}

/**
 * Maps the backend status into claimed/unclaimed per game. Games whose status
 * is missing are left out rather than defaulting to unclaimed, so the UI can
 * tell "not claimed" from "not known yet".
 */
export function extractClaimStatus(status: AllDailyRewardStatus): Map<GameId, boolean> {
  const map = new Map<GameId, boolean>();
  for (const [gameId, data] of Object.entries(status.games ?? {})) {
    if (data.info?.is_signed != null) {
      map.set(gameId as GameId, data.info.is_signed);
    }
  }
  return map;
}

/** UTC+8 offset in milliseconds (all HoYoLab games reset at midnight UTC+8). */
const UTC8_OFFSET_MS = 8 * 3_600_000;

/** The calendar date in UTC+8, the boundary the games reset on. */
export function utc8DateString(nowMs: number): string {
  return new Date(nowMs + UTC8_OFFSET_MS).toISOString().slice(0, 10);
}
