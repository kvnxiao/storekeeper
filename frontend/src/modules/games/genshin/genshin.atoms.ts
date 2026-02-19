import { atom } from "jotai";
import type { CoreAtoms } from "@/modules/core/core.atoms";
import { atomFormattedTime } from "@/modules/games/atomFormattedTime";
import { atomResourceSelector } from "@/modules/games/atomResourceSelector";
import { GenshinResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import {
  isCooldownResource,
  isExpeditionResource,
  isStaminaResource,
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
    isStaminaResource,
  );

  readonly resinTime = atomFormattedTime(
    () => this.core,
    (get) => get(this.resin)?.fullAt,
  );

  readonly parametricTransformer = atomResourceSelector(
    () => this.core,
    GameId.GenshinImpact,
    GenshinResource.ParametricTransformer,
    isCooldownResource,
  );

  readonly parametricTransformerTime = atomFormattedTime(
    () => this.core,
    (get) => get(this.parametricTransformer)?.readyAt,
  );

  readonly realmCurrency = atomResourceSelector(
    () => this.core,
    GameId.GenshinImpact,
    GenshinResource.RealmCurrency,
    isStaminaResource,
  );

  readonly realmCurrencyTime = atomFormattedTime(
    () => this.core,
    (get) => get(this.realmCurrency)?.fullAt,
  );

  readonly expeditions = atomResourceSelector(
    () => this.core,
    GameId.GenshinImpact,
    GenshinResource.Expeditions,
    isExpeditionResource,
  );

  readonly expeditionsTime = atomFormattedTime(
    () => this.core,
    (get) => get(this.expeditions)?.earliestFinishAt,
  );

  readonly expeditionsReady = atom((get) => {
    const nowMs = get(this.core.tick);
    const data = get(this.expeditions);
    if (!data) return false;
    if (data.currentExpeditions === 0) return false;

    return isPastDateTime(data.earliestFinishAt, nowMs);
  });
}
