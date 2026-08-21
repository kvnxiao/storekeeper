import { useQuery } from "@tanstack/solid-query";
import { createMemo } from "solid-js";
import { core } from "@/modules/core/core.state";
import { GenshinResource } from "@/modules/games/games.constants";
import {
  createCooldownResource,
  createFormattedTime,
  createStaminaResource,
} from "@/modules/games/games.primitives";
import { GameId } from "@/modules/games/games.types";
import { selectResource } from "@/modules/games/games.utils";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import { type CooldownResource, isExpeditionResource } from "@/modules/resources/resources.types";
import { isPastDateTime } from "@/modules/resources/resources.utils";

/** Reactive Genshin Impact resource selectors over the resources query. */
export function createGenshinResources() {
  const query = useQuery(() => resourcesQueryOptions());
  const resources = () => query.data;

  const [resin, resinTime] = createStaminaResource(
    resources,
    () => GameId.GenshinImpact,
    () => GenshinResource.Resin,
  );
  const [realmCurrency, realmCurrencyTime] = createStaminaResource(
    resources,
    () => GameId.GenshinImpact,
    () => GenshinResource.RealmCurrency,
  );
  const [parametricTransformer, parametricTransformerTime] = createCooldownResource(
    resources,
    () => GameId.GenshinImpact,
    () => GenshinResource.ParametricTransformer,
  );

  const expeditions = createMemo(() =>
    selectResource(
      resources(),
      GameId.GenshinImpact,
      GenshinResource.Expeditions,
      isExpeditionResource,
    ),
  );
  const expeditionsTime = createFormattedTime(() => expeditions()?.earliestFinishAt);

  const expeditionsReady = createMemo(() => {
    const data = expeditions();
    if (!data || data.currentExpeditions === 0) {
      return false;
    }
    return isPastDateTime(data.earliestFinishAt, core.tick());
  });

  // Expeditions render as a cooldown: ready when every dispatched expedition
  // has finished.
  const expeditionsCooldown = createMemo<CooldownResource | null>(() => {
    const data = expeditions();
    return data ? { isReady: expeditionsReady(), readyAt: data.earliestFinishAt } : null;
  });

  return {
    resin,
    resinTime,
    parametricTransformer,
    parametricTransformerTime,
    realmCurrency,
    realmCurrencyTime,
    expeditionsCooldown,
    expeditionsTime,
  };
}
