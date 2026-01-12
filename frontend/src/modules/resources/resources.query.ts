import type { MutationOptions, QueryOptions } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import type { AllResources } from "@/modules/resources/resources.types";

/** Query options for fetching all resources from Tauri backend */
export const resourcesQueryOptions: QueryOptions<AllResources> = {
  queryKey: ["resources"],
  queryFn: async () => invoke<AllResources>("get_all_resources"),
  retry: false,
  refetchOnWindowFocus: true,
};

/** Mutation options for refreshing resources */
export const refreshResourcesMutationOptions: MutationOptions<
  AllResources,
  Error,
  void
> = {
  mutationKey: ["refresh-resources"],
  mutationFn: async () => invoke<AllResources>("refresh_resources"),
};
