import { useQuery } from "@tanstack/solid-query";
import { createMemo } from "solid-js";
import { HsrResource } from "@/modules/games/games.constants";
import { createFormattedTime } from "@/modules/games/games.primitives";
import { GameId } from "@/modules/games/games.types";
import { selectResource } from "@/modules/games/games.utils";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import { isStaminaResource } from "@/modules/resources/resources.types";

/** Reactive Honkai: Star Rail resource selectors over the resources query. */
export function createHsrResources() {
  const resourcesQuery = useQuery(() => resourcesQueryOptions());

  const trailblazePower = createMemo(() =>
    selectResource(
      resourcesQuery.data,
      GameId.HonkaiStarRail,
      HsrResource.TrailblazePower,
      isStaminaResource,
    ),
  );
  const trailblazePowerTime = createFormattedTime(() => trailblazePower()?.fullAt);

  return { trailblazePower, trailblazePowerTime };
}
