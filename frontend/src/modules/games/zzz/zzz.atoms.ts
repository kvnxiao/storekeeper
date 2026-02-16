import type { CoreAtoms } from "@/modules/core/core.atoms";
import { createResourceSelector } from "@/modules/games/games.atoms";
import { ZzzResource } from "@/modules/games/games.constants";

// =============================================================================
// ZzzAtoms Class
// =============================================================================

export class ZzzAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly battery = createResourceSelector(
    () => this.core,
    "ZENLESS_ZONE_ZERO",
    ZzzResource.Battery,
  );
}
