import { mutationOptions, queryOptions } from "@tanstack/solid-query";
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
 * events wired in `core.init()` and by the reset watcher in
 * `daily-rewards.state.ts`.
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

/** Mutation options for manually claiming a game's daily reward. */
export function claimDailyRewardMutationOptions() {
  return mutationOptions({
    mutationKey: ["claim-daily-reward"],
    mutationFn: async (gameId: GameId) => invoke("claim_daily_reward_for_game", { gameId }),
    onSuccess: () => invalidateDailyRewardStatus(),
    onError: (error) => console.error("Failed to claim daily reward:", error),
  });
}
