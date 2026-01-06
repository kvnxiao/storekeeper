import { invoke } from "@tauri-apps/api/core";
import { atomWithMutation, atomWithQuery } from "jotai-tanstack-query";

import type { AllResources } from "@/types";

/** Fetch all resources from Tauri backend */
export const resourcesAtom = atomWithQuery(() => ({
  queryKey: ["resources"],
  queryFn: async () => {
    const result = await invoke<AllResources>("get_all_resources");
    return result;
  },
  retry: false,
  refetchOnWindowFocus: true,
}));

/** Refresh resources mutation */
export const refreshResourcesAtom = atomWithMutation(() => ({
  mutationKey: ["refresh-resources"],
  mutationFn: async () => {
    const result = await invoke<AllResources>("refresh_resources");
    return result;
  },
}));
