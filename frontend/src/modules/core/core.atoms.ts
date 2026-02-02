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
// CoreAtoms Class
// =============================================================================

export class CoreAtoms {
  // ---------------------------------------------------------------------------
  // Tick system - updates every minute for real-time countdown display
  // ---------------------------------------------------------------------------

  readonly tickBase = atom<number>(Date.now());
  readonly tickRestartSignal = atom<number>(0);

  private readonly tickEffect = atomEffect((get, set) => {
    get(this.tickRestartSignal);
    set(this.tickBase, Date.now());

    const interval = setInterval(() => {
      set(this.tickBase, Date.now());
    }, 60_000);

    return () => clearInterval(interval);
  });

  readonly tick = atom((get) => {
    get(this.tickEffect);
    return get(this.tickBase);
  });

  readonly refreshTick = atom(null, (get, set) => {
    set(this.tickBase, Date.now());
    set(this.tickRestartSignal, get(this.tickRestartSignal) + 1);
  });

  // ---------------------------------------------------------------------------
  // Resources query & mutation
  // ---------------------------------------------------------------------------

  readonly resourcesQuery = atomWithQuery(() => resourcesQueryOptions());
  readonly refreshResources = atomWithMutation(() =>
    refreshResourcesMutationOptions(),
  );

  // ---------------------------------------------------------------------------
  // Event listener - listens for backend resource updates
  // ---------------------------------------------------------------------------

  private readonly resourcesEventEffect = atomEffect((_get, set) => {
    const unlisten = listen<AllResources>("resources-updated", (event) => {
      queryClient.setQueryData(["resources"], event.payload);
      set(this.refreshTick);
    });

    return () => {
      void unlisten.then((fn) => fn());
    };
  });

  readonly resourcesEventListener = atom((get) => {
    get(this.resourcesEventEffect);
  });
}
