import type { CoreAtoms } from "@/modules/core/core.atoms";
import { atomResourceSelector } from "@/modules/games/atomResourceSelector";
import { WuwaResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";

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
}
