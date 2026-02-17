import type { CoreAtoms } from "@/modules/core/core.atoms";
import { atomResourceSelector } from "@/modules/games/atomResourceSelector";
import { ZzzResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";

// =============================================================================
// ZzzAtoms Class
// =============================================================================

export class ZzzAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly battery = atomResourceSelector(
    () => this.core,
    GameId.ZenlessZoneZero,
    ZzzResource.Battery,
  );
}
