import { atom } from "jotai";
import type { CoreAtoms } from "@/modules/core/core.atoms";
import { createResourceSelector } from "@/modules/games/games.atoms";
import { GenshinResource } from "@/modules/games/games.constants";
import type { ExpeditionResource } from "@/modules/resources/resources.types";
import { isPastDateTime } from "@/modules/resources/resources.utils";

// =============================================================================
// GenshinAtoms Class
// =============================================================================

export class GenshinAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly resin = createResourceSelector(
    () => this.core,
    "GENSHIN_IMPACT",
    GenshinResource.Resin,
  );

  readonly parametricTransformer = createResourceSelector(
    () => this.core,
    "GENSHIN_IMPACT",
    GenshinResource.ParametricTransformer,
  );

  readonly realmCurrency = createResourceSelector(
    () => this.core,
    "GENSHIN_IMPACT",
    GenshinResource.RealmCurrency,
  );

  readonly expeditions = createResourceSelector(
    () => this.core,
    "GENSHIN_IMPACT",
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
