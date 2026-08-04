import { useQuery } from "@tanstack/solid-query";
import { createMemo } from "solid-js";
import { core } from "@/modules/core/core.state";
import { GenshinResource } from "@/modules/games/games.constants";
import { createFormattedTime } from "@/modules/games/games.primitives";
import { GameId } from "@/modules/games/games.types";
import { selectResource } from "@/modules/games/games.utils";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import {
  isCooldownResource,
  isExpeditionResource,
  isStaminaResource,
} from "@/modules/resources/resources.types";
import { isPastDateTime } from "@/modules/resources/resources.utils";

/** Reactive Genshin Impact resource selectors over the resources query. */
export function createGenshinResources() {
  const resourcesQuery = useQuery(() => resourcesQueryOptions());

  const resin = createMemo(() =>
    selectResource(
      resourcesQuery.data,
      GameId.GenshinImpact,
      GenshinResource.Resin,
      isStaminaResource,
    ),
  );
  const resinTime = createFormattedTime(() => resin()?.fullAt);

  const parametricTransformer = createMemo(() =>
    selectResource(
      resourcesQuery.data,
      GameId.GenshinImpact,
      GenshinResource.ParametricTransformer,
      isCooldownResource,
    ),
  );
  const parametricTransformerTime = createFormattedTime(() => parametricTransformer()?.readyAt);

  const realmCurrency = createMemo(() =>
    selectResource(
      resourcesQuery.data,
      GameId.GenshinImpact,
      GenshinResource.RealmCurrency,
      isStaminaResource,
    ),
  );
  const realmCurrencyTime = createFormattedTime(() => realmCurrency()?.fullAt);

  const expeditions = createMemo(() =>
    selectResource(
      resourcesQuery.data,
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

  return {
    resin,
    resinTime,
    parametricTransformer,
    parametricTransformerTime,
    realmCurrency,
    realmCurrencyTime,
    expeditions,
    expeditionsTime,
    expeditionsReady,
  };
}
