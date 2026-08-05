import { queryOptions } from "@tanstack/solid-query";
import { invoke } from "@tauri-apps/api/core";
import { queryClient } from "@/modules/core/core.queryClient";
import type { GameId } from "@/modules/games/games.types";

/** Backend response shape (private, only used for extraction) */
interface AllDailyRewardStatus {
  games?: Record<string, { info?: { is_signed?: boolean } }>;
  lastChecked?: string;
}

function extractClaimStatus(status: AllDailyRewardStatus): Map<GameId, boolean> {
  const map = new Map<GameId, boolean>();
  for (const [gameId, data] of Object.entries(status.games ?? {})) {
    if (data.info?.is_signed != null) {
      map.set(gameId as GameId, data.info.is_signed);
    }
  }
  return map;
}

/**
 * Query options for per-game daily-reward claim status.
 *
 * Never goes stale on its own: the cache is updated by the daily-reward
 * events wired in `core.init()` and by the daily reset watcher below.
 */
export function dailyRewardStatusQueryOptions() {
  return queryOptions({
    queryKey: ["daily-reward-status"],
    queryFn: async () =>
      extractClaimStatus(await invoke<AllDailyRewardStatus>("get_daily_reward_status")),
    staleTime: Number.POSITIVE_INFINITY,
  });
}

/** Forces a backend refresh from the game APIs and updates the cache. */
export async function refreshDailyRewardStatus(): Promise<void> {
  const status = await invoke<AllDailyRewardStatus>("refresh_daily_reward_status");
  queryClient.setQueryData(dailyRewardStatusQueryOptions().queryKey, extractClaimStatus(status));
}

/** Re-reads the backend's already-fresh status (cheap; used after claim events). */
export async function invalidateDailyRewardStatus(): Promise<void> {
  await queryClient.invalidateQueries({ queryKey: dailyRewardStatusQueryOptions().queryKey });
}

/** UTC+8 offset in milliseconds (all HoYoLab games reset at midnight UTC+8). */
const UTC8_OFFSET_MS = 8 * 3_600_000;

function getUtc8DateString(): string {
  return new Date(Date.now() + UTC8_OFFSET_MS).toISOString().slice(0, 10);
}

let lastUtc8Date = getUtc8DateString();

/**
 * Detects the UTC+8 date rollover and re-fetches claim status after a buffer
 * for game server reset propagation. Driven by the core minute tick.
 */
export function checkDailyReset(): void {
  const currentDate = getUtc8DateString();
  if (currentDate === lastUtc8Date) {
    return;
  }
  lastUtc8Date = currentDate;
  setTimeout(() => {
    refreshDailyRewardStatus().catch(console.error);
  }, 60_000);
}
