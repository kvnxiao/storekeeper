import type { CoreAtoms } from "@/modules/core/core.atoms";
import { atomResourceSelector } from "@/modules/games/atomResourceSelector";
import { HsrResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";

// =============================================================================
// HsrAtoms Class
// =============================================================================

export class HsrAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly trailblazePower = atomResourceSelector(
    () => this.core,
    GameId.HonkaiStarRail,
    HsrResource.TrailblazePower,
  );
}
