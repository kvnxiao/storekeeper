import type { CoreAtoms } from "@/modules/core/core.atoms";
import { atomFormattedTime } from "@/modules/games/atomFormattedTime";
import { atomResourceSelector } from "@/modules/games/atomResourceSelector";
import { HsrResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { isStaminaResource } from "@/modules/resources/resources.types";

// =============================================================================
// HsrAtoms Class
// =============================================================================

export class HsrAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly trailblazePower = atomResourceSelector(
    () => this.core,
    GameId.HonkaiStarRail,
    HsrResource.TrailblazePower,
    isStaminaResource,
  );

  readonly trailblazePowerTime = atomFormattedTime(
    () => this.core,
    (get) => get(this.trailblazePower)?.fullAt,
  );
}
