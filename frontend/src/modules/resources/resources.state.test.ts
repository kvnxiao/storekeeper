import { createRoot } from "solid-js";
import { describe, expect, it, vi } from "vite-plus/test";
import { queryClient } from "@/modules/core/core.queryClient";
import { REFRESH_RESOURCES_MUTATION_KEY } from "@/modules/resources/resources.query";
import { createResourcesState } from "@/modules/resources/resources.state";

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

    await vi.waitFor(() => expect(state.isRefreshing()).toBe(true));

    settle?.();
    await executed;
    await vi.waitFor(() => expect(state.isRefreshing()).toBe(false));

    dispose();
  });
});
