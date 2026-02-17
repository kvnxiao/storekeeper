import { atom } from "jotai";
import type { CoreAtoms } from "@/modules/core/core.atoms";
import { atomResourceSelector } from "@/modules/games/atomResourceSelector";
import { GenshinResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import type { ExpeditionResource } from "@/modules/resources/resources.types";
import { isPastDateTime } from "@/modules/resources/resources.utils";

// =============================================================================
// GenshinAtoms Class
// =============================================================================

export class GenshinAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly resin = atomResourceSelector(
    () => this.core,
    GameId.GenshinImpact,
    GenshinResource.Resin,
  );

  readonly parametricTransformer = atomResourceSelector(
    () => this.core,
    GameId.GenshinImpact,
    GenshinResource.ParametricTransformer,
  );

  readonly realmCurrency = atomResourceSelector(
    () => this.core,
    GameId.GenshinImpact,
    GenshinResource.RealmCurrency,
  );

  readonly expeditions = atomResourceSelector(
    () => this.core,
    GameId.GenshinImpact,
    GenshinResource.Expeditions,
  );

  readonly expeditionsReady = atom((get) => {
    get(this.core.tick);
    const resource = get(this.expeditions);
    if (!resource) return false;

    const data = resource.data as ExpeditionResource;
    if (!("currentExpeditions" in data)) return false;
    if (data.currentExpeditions === 0) return false;

    return isPastDateTime(data.earliestFinishAt, Date.now());
  });
}
