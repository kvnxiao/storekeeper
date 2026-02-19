import { atom } from "jotai";
import type { CoreAtoms } from "@/modules/core/core.atoms";
import { atomFormattedTime } from "@/modules/games/atomFormattedTime";
import { atomResourceSelector } from "@/modules/games/atomResourceSelector";
import { GenshinResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import type {
  CooldownResource,
  ExpeditionResource,
  StaminaResource,
} from "@/modules/resources/resources.types";
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

  readonly resinTime = atomFormattedTime(
    () => this.core,
    (get) => (get(this.resin)?.data as StaminaResource | undefined)?.fullAt,
  );

  readonly parametricTransformer = atomResourceSelector(
    () => this.core,
    GameId.GenshinImpact,
    GenshinResource.ParametricTransformer,
  );

  readonly parametricTransformerTime = atomFormattedTime(
    () => this.core,
    (get) =>
      (get(this.parametricTransformer)?.data as CooldownResource | undefined)
        ?.readyAt,
  );

  readonly realmCurrency = atomResourceSelector(
    () => this.core,
    GameId.GenshinImpact,
    GenshinResource.RealmCurrency,
  );

  readonly realmCurrencyTime = atomFormattedTime(
    () => this.core,
    (get) =>
      (get(this.realmCurrency)?.data as StaminaResource | undefined)?.fullAt,
  );

  readonly expeditions = atomResourceSelector(
    () => this.core,
    GameId.GenshinImpact,
    GenshinResource.Expeditions,
  );

  readonly expeditionsTime = atomFormattedTime(
    () => this.core,
    (get) =>
      (get(this.expeditions)?.data as ExpeditionResource | undefined)
        ?.earliestFinishAt,
  );

  readonly expeditionsReady = atom((get) => {
    const nowMs = get(this.core.tick);
    const resource = get(this.expeditions);
    if (!resource) return false;

    const data = resource.data as ExpeditionResource;
    if (!("currentExpeditions" in data)) return false;
    if (data.currentExpeditions === 0) return false;

    return isPastDateTime(data.earliestFinishAt, nowMs);
  });
}
