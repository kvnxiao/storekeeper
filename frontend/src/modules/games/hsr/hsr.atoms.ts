import type { CoreAtoms } from "@/modules/core/core.atoms";
import { createResourceSelector } from "@/modules/games/games.atoms";
import { HsrResource } from "@/modules/games/games.constants";

// =============================================================================
// HsrAtoms Class
// =============================================================================

export class HsrAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly trailblazePower = createResourceSelector(
    () => this.core,
    "HONKAI_STAR_RAIL",
    HsrResource.TrailblazePower,
  );
}
