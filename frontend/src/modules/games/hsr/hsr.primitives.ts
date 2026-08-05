import { useQuery } from "@tanstack/solid-query";
import { HsrResource } from "@/modules/games/games.constants";
import { createStaminaResource } from "@/modules/games/games.primitives";
import { GameId } from "@/modules/games/games.types";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";

/** Reactive Honkai: Star Rail resource selectors over the resources query. */
export function createHsrResources() {
  const query = useQuery(() => resourcesQueryOptions());
  const [trailblazePower, trailblazePowerTime] = createStaminaResource(
    () => query.data,
    GameId.HonkaiStarRail,
    HsrResource.TrailblazePower,
  );

  return { trailblazePower, trailblazePowerTime };
}
