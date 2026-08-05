import { useIsMutating } from "@tanstack/solid-query";
import type { Accessor } from "solid-js";
import { queryClient } from "@/modules/core/core.queryClient";
import { REFRESH_RESOURCES_MUTATION_KEY } from "@/modules/resources/resources.query";
import { resourcesState } from "@/modules/resources/resources.state";

/**
 * True while any resource refresh is in flight: backend-driven refreshes
 * (event signal) or the manual refresh mutation, whose pending state covers
 * the gap before the backend emits `refresh-started`.
 */
export function createIsRefreshing(): Accessor<boolean> {
  const pendingRefreshes = useIsMutating(
    () => ({ mutationKey: REFRESH_RESOURCES_MUTATION_KEY }),
    () => queryClient,
  );
  return () => resourcesState.isRefreshing() || pendingRefreshes() > 0;
}
