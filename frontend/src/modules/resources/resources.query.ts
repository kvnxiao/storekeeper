import { mutationOptions, queryOptions } from "@tanstack/solid-query";
import { invoke } from "@tauri-apps/api/core";
import type { AllResources } from "@/modules/resources/resources.types";

export const REFRESH_RESOURCES_MUTATION_KEY = ["refresh-resources"] as const;

/** Query options for fetching all resources from Tauri backend */
export function resourcesQueryOptions() {
  return queryOptions({
    queryKey: ["resources"],
    queryFn: async () => invoke<AllResources>("get_all_resources"),
    // Backend events keep this cache fresh; focus/navigation must not refetch.
    staleTime: Number.POSITIVE_INFINITY,
    // Only the dashboard observes this, so the default collection window drops
    // the snapshot while the user sits in settings and brings the dashboard
    // back on skeletons.
    gcTime: Number.POSITIVE_INFINITY,
  });
}

/**
 * Mutation options for triggering a refresh; the mutation only tracks progress.
 *
 * The refreshed snapshot arrives on `resources-updated` rather than from the
 * command response: responses and events reach the webview on separate
 * channels, so writing both would let their arrival order decide what renders.
 */
export function refreshResourcesMutationOptions() {
  return mutationOptions({
    mutationKey: REFRESH_RESOURCES_MUTATION_KEY,
    mutationFn: async () => {
      await invoke("refresh_resources");
    },
  });
}
