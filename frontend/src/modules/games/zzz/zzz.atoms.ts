import type { CoreAtoms } from "@/modules/core/core.atoms";
import { atomFormattedTime } from "@/modules/games/atomFormattedTime";
import { atomResourceSelector } from "@/modules/games/atomResourceSelector";
import { ZzzResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { isStaminaResource } from "@/modules/resources/resources.types";

// =============================================================================
// ZzzAtoms Class
// =============================================================================

export class ZzzAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly battery = atomResourceSelector(
    () => this.core,
    GameId.ZenlessZoneZero,
    ZzzResource.Battery,
    isStaminaResource,
  );

  readonly batteryTime = atomFormattedTime(
    () => this.core,
    (get) => get(this.battery)?.fullAt,
  );
}
