import { listen } from "@tauri-apps/api/event";
import { atom } from "jotai";
import { atomEffect } from "jotai-effect";
import { atomWithMutation, atomWithQuery } from "jotai-tanstack-query";

import { queryClient } from "@/modules/core/core.queryClient";
import {
  refreshResourcesMutationOptions,
  resourcesQueryOptions,
} from "@/modules/resources/resources.query";
import type { AllResources } from "@/modules/resources/resources.types";

// =============================================================================
// Query & Mutation Atoms
// =============================================================================

/** Fetch all resources from Tauri backend */
export const resourcesQueryAtom = atomWithQuery(() => resourcesQueryOptions());

/** Refresh resources mutation */
export const refreshResourcesMutationAtom = atomWithMutation(() =>
  refreshResourcesMutationOptions(),
);

// =============================================================================
// Event Listener
// =============================================================================

/** Effect atom that listens for backend resource updates */
const resourcesEventEffect = atomEffect(() => {
  const unlisten = listen<AllResources>("resources-updated", (event) => {
    queryClient.setQueryData(["resources"], event.payload);
  });

  return () => {
    void unlisten.then((fn) => fn());
  };
});

/** Subscribe to enable resource update listening */
export const resourcesEventListenerAtom = atom((get) => {
  get(resourcesEventEffect);
});
