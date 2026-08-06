import { createRoot } from "solid-js";
import { describe, expect, it, vi } from "vite-plus/test";
import { queryClient } from "@/modules/core/core.queryClient";
import { GameId } from "@/modules/games/games.types";
import { REFRESH_RESOURCES_MUTATION_KEY } from "@/modules/resources/resources.query";
import { createResourcesState } from "@/modules/resources/resources.state";

const WAIT_TIMEOUT_MS = 3_000;

function withState<T>(run: (state: ReturnType<typeof createResourcesState>) => T): T {
  return createRoot((dispose) => {
    const result = run(createResourcesState());
    dispose();
    return result;
  });
}

describe("resources state", () => {
  it("is idle until a refresh starts", () => {
    expect(withState((state) => state.isRefreshing())).toBe(false);
  });

  it("follows the backend refresh events", () => {
    withState((state) => {
      state.refreshStarted();
      expect(state.isRefreshing()).toBe(true);
      state.refreshSettled();
      expect(state.isRefreshing()).toBe(false);
    });
  });

  it("stops masking a game as soon as its own resources land", () => {
    withState((state) => {
      state.refreshStarted();
      state.gameSettled(GameId.GenshinImpact);

      expect(state.isGameRefreshing(GameId.GenshinImpact)).toBe(false);
      expect(state.isGameRefreshing(GameId.HonkaiStarRail)).toBe(true);
      expect(state.isRefreshing()).toBe(true);
    });
  });

  it("masks a settled game again only once the next refresh starts", () => {
    withState((state) => {
      state.refreshStarted();
      state.gameSettled(GameId.GenshinImpact);
      state.refreshSettled();
      expect(state.isGameRefreshing(GameId.GenshinImpact)).toBe(false);

      state.refreshStarted();
      expect(state.isGameRefreshing(GameId.GenshinImpact)).toBe(true);
    });
  });

  it("is refreshing while the manual refresh mutation is pending", async () => {
    let settle: (() => void) | undefined;
    const pending = new Promise<void>((resolve) => {
      settle = resolve;
    });

    const { state, dispose } = createRoot((disposeRoot) => ({
      state: createResourcesState(),
      dispose: disposeRoot,
    }));

    const mutation = queryClient.getMutationCache().build(queryClient, {
      mutationKey: REFRESH_RESOURCES_MUTATION_KEY,
      mutationFn: async () => pending,
    });
    const executed = mutation.execute(undefined);

    await vi.waitFor(() => expect(state.isRefreshing()).toBe(true), {
      timeout: WAIT_TIMEOUT_MS,
    });

    settle?.();
    await executed;
    await vi.waitFor(() => expect(state.isRefreshing()).toBe(false), {
      timeout: WAIT_TIMEOUT_MS,
    });

    dispose();
  });
});
