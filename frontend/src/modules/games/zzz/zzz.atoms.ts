import { atom } from "jotai";

import type { CoreAtoms } from "@/modules/core/core.atoms";

// =============================================================================
// ZzzAtoms Class
// =============================================================================

export class ZzzAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly battery = atom((get) => {
    const { data } = get(this.core.resourcesQuery);
    return (
      data?.games?.ZENLESS_ZONE_ZERO?.find((r) => r.type === "battery") ?? null
    );
  });
}
