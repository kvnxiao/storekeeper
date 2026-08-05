import { HsrResource } from "@/modules/games/games.constants";
import { createStaminaResource } from "@/modules/games/games.primitives";
import { GameId } from "@/modules/games/games.types";

/** Reactive Honkai: Star Rail resource selectors over the resources query. */
export function createHsrResources() {
  const [trailblazePower, trailblazePowerTime] = createStaminaResource(
    GameId.HonkaiStarRail,
    HsrResource.TrailblazePower,
  );

  return { trailblazePower, trailblazePowerTime };
}
