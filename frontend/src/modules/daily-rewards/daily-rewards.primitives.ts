import { useQuery } from "@tanstack/solid-query";
import type { Accessor } from "solid-js";
import { dailyRewardStatusQueryOptions } from "@/modules/daily-rewards/daily-rewards.query";
import type { GameId } from "@/modules/games/games.types";

/** Reactive daily-reward claim status for a game; `null` while unknown. */
export function createClaimStatus(gameId: GameId): Accessor<boolean | null> {
  const query = useQuery(() => dailyRewardStatusQueryOptions());
  return () => query.data?.get(gameId) ?? null;
}
