import type { CoreAtoms } from "@/modules/core/core.atoms";
import { atomFormattedTime } from "@/modules/games/atomFormattedTime";
import { atomResourceSelector } from "@/modules/games/atomResourceSelector";
import { WuwaResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { isStaminaResource } from "@/modules/resources/resources.types";

// =============================================================================
// WuwaAtoms Class
// =============================================================================

export class WuwaAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly waveplates = atomResourceSelector(
    () => this.core,
    GameId.WutheringWaves,
    WuwaResource.Waveplates,
    isStaminaResource,
  );

  readonly waveplatesTime = atomFormattedTime(
    () => this.core,
    (get) => get(this.waveplates)?.fullAt,
  );
}
