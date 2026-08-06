import { useIsMutating } from "@tanstack/solid-query";
import { createRoot, createSignal } from "solid-js";
import { queryClient } from "@/modules/core/core.queryClient";
import type { GameId } from "@/modules/games/games.types";
import { REFRESH_RESOURCES_MUTATION_KEY } from "@/modules/resources/resources.query";

export function createResourcesState() {
  const [eventRefreshing, setEventRefreshing] = createSignal(false);
  const [settledGames, setSettledGames] = createSignal<ReadonlySet<GameId>>(new Set<GameId>());

  const pendingRefreshes = useIsMutating(
    () => ({ mutationKey: REFRESH_RESOURCES_MUTATION_KEY }),
    () => queryClient,
  );

  /**
   * True while any resource refresh is in flight. The mutation's pending state
   * covers the gap before the backend emits `refresh-started`; the event flag
   * covers refreshes the backend starts on its own.
   */
  const isRefreshing = () => eventRefreshing() || pendingRefreshes() > 0;

  /**
   * True while a refresh is in flight and this game's data has not landed.
   *
   * The backend fetches the games one provider at a time and announces each one
   * as it completes, so a game that is already back must not keep waiting on
   * the slowest one. The settled set is cleared when the next refresh starts
   * rather than when this one ends, which keeps the games that did land from
   * flashing again while the command response trails the snapshot event.
   */
  function isGameRefreshing(gameId: GameId): boolean {
    return isRefreshing() && !settledGames().has(gameId);
  }

  function refreshStarted(): void {
    setSettledGames(new Set<GameId>());
    setEventRefreshing(true);
  }

  function gameSettled(gameId: GameId): void {
    setSettledGames((games) => new Set(games).add(gameId));
  }

  function refreshSettled(): void {
    setEventRefreshing(false);
  }

  return { isRefreshing, isGameRefreshing, refreshStarted, gameSettled, refreshSettled };
}

/**
 * Refresh state for the whole app: one mutation-cache subscription, read
 * directly by the views that shimmer or disable on it. The event flag is
 * written by the backend event listeners in `core.init()`.
 */
export const resourcesState = createRoot(createResourcesState);
