import { atom } from "jotai";
import type { CoreAtoms } from "@/modules/core/core.atoms";
import type { ExpeditionResource } from "@/modules/resources/resources.types";
import { isPastDateTime } from "@/modules/resources/resources.utils";

// =============================================================================
// GenshinAtoms Class
// =============================================================================

export class GenshinAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly resin = atom((get) => {
    const { data } = get(this.core.resourcesQuery);
    return data?.games?.GENSHIN_IMPACT?.find((r) => r.type === "resin") ?? null;
  });

  readonly parametricTransformer = atom((get) => {
    const { data } = get(this.core.resourcesQuery);
    return (
      data?.games?.GENSHIN_IMPACT?.find(
        (r) => r.type === "parametric_transformer",
      ) ?? null
    );
  });

  readonly realmCurrency = atom((get) => {
    const { data } = get(this.core.resourcesQuery);
    return (
      data?.games?.GENSHIN_IMPACT?.find((r) => r.type === "realm_currency") ??
      null
    );
  });

  readonly expeditions = atom((get) => {
    const { data } = get(this.core.resourcesQuery);
    return (
      data?.games?.GENSHIN_IMPACT?.find((r) => r.type === "expeditions") ?? null
    );
  });

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
