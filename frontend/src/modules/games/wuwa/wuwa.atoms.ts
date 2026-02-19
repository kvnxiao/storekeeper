import type { CoreAtoms } from "@/modules/core/core.atoms";
import { atomFormattedTime } from "@/modules/games/atomFormattedTime";
import { atomResourceSelector } from "@/modules/games/atomResourceSelector";
import { WuwaResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import type { StaminaResource } from "@/modules/resources/resources.types";

// =============================================================================
// WuwaAtoms Class
// =============================================================================

export class WuwaAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly waveplates = atomResourceSelector(
    () => this.core,
    GameId.WutheringWaves,
    WuwaResource.Waveplates,
  );

  readonly waveplatesTime = atomFormattedTime(
    () => this.core,
    (get) =>
      (get(this.waveplates)?.data as StaminaResource | undefined)?.fullAt,
  );
}
