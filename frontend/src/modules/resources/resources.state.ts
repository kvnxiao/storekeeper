import { useIsMutating } from "@tanstack/solid-query";
import { createRoot, createSignal } from "solid-js";
import { queryClient } from "@/modules/core/core.queryClient";
import { REFRESH_RESOURCES_MUTATION_KEY } from "@/modules/resources/resources.query";

export function createResourcesState() {
  const [eventRefreshing, setEventRefreshing] = createSignal(false);

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

  function refreshStarted(): void {
    setEventRefreshing(true);
  }

  function refreshSettled(): void {
    setEventRefreshing(false);
  }

  return { isRefreshing, refreshStarted, refreshSettled };
}

/**
 * Refresh state for the whole app: one mutation-cache subscription, read
 * directly by the views that shimmer or disable on it. The event flag is
 * written by the backend event listeners in `core.init()`.
 */
export const resourcesState = createRoot(createResourcesState);
