import { mutationOptions, type QueryClient, queryOptions } from "@tanstack/solid-query";
import { invoke } from "@tauri-apps/api/core";
import type { AllResources } from "@/modules/resources/resources.types";

export const REFRESH_RESOURCES_MUTATION_KEY = ["refresh-resources"] as const;

/** Query options for fetching all resources from Tauri backend */
export function resourcesQueryOptions() {
  return queryOptions({
    queryKey: ["resources"],
    queryFn: async () => invoke<AllResources>("get_all_resources"),
  });
}

/** Mutation options for refreshing resources; the result becomes the cached snapshot */
export function refreshResourcesMutationOptions(queryClient: QueryClient) {
  return mutationOptions({
    mutationKey: REFRESH_RESOURCES_MUTATION_KEY,
    mutationFn: async () => invoke<AllResources>("refresh_resources"),
    onSuccess: (data) => {
      queryClient.setQueryData(resourcesQueryOptions().queryKey, data);
    },
  });
}
