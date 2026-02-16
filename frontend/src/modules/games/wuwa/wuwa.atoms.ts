import type { CoreAtoms } from "@/modules/core/core.atoms";
import { createResourceSelector } from "@/modules/games/games.atoms";
import { WuwaResource } from "@/modules/games/games.constants";

// =============================================================================
// WuwaAtoms Class
// =============================================================================

export class WuwaAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly waveplates = createResourceSelector(
    () => this.core,
    "WUTHERING_WAVES",
    WuwaResource.Waveplates,
  );
}
