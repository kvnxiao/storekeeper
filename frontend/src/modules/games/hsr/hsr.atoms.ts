import { atom } from "jotai";
import type { CoreAtoms } from "@/modules/core/core.atoms";

// =============================================================================
// HsrAtoms Class
// =============================================================================

export class HsrAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly trailblazePower = atom((get) => {
    const { data } = get(this.core.resourcesQuery);
    return (
      data?.games?.HONKAI_STAR_RAIL?.find(
        (r) => r.type === "trailblaze_power",
      ) ?? null
    );
  });
}
