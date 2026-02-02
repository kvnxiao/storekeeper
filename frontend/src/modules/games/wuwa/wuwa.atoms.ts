import { atom } from "jotai";

import type { CoreAtoms } from "@/modules/core/core.atoms";

// =============================================================================
// WuwaAtoms Class
// =============================================================================

export class WuwaAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly waveplates = atom((get) => {
    const { data } = get(this.core.resourcesQuery);
    return (
      data?.games?.WUTHERING_WAVES?.find((r) => r.type === "waveplates") ?? null
    );
  });
}
