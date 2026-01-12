import { mutationOptions, queryOptions } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import type { AllResources } from "@/modules/resources/resources.types";

/** Query options for fetching all resources from Tauri backend */
export function resourcesQueryOptions() {
  return queryOptions({
    queryKey: ["resources"],
    queryFn: async () => invoke<AllResources>("get_all_resources"),
    retry: false,
    refetchOnWindowFocus: true,
  });
}

/** Mutation options for refreshing resources */
export function refreshResourcesMutationOptions() {
  return mutationOptions({
    mutationKey: ["refresh-resources"],
    mutationFn: async () => invoke<AllResources>("refresh_resources"),
  });
}
