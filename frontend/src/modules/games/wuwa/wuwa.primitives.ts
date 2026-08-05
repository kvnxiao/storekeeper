import { useQuery } from "@tanstack/solid-query";
import { WuwaResource } from "@/modules/games/games.constants";
import { createStaminaResource } from "@/modules/games/games.primitives";
import { GameId } from "@/modules/games/games.types";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";

/** Reactive Wuthering Waves resource selectors over the resources query. */
export function createWuwaResources() {
  const query = useQuery(() => resourcesQueryOptions());
  const [waveplates, waveplatesTime] = createStaminaResource(
    () => query.data,
    GameId.WutheringWaves,
    WuwaResource.Waveplates,
  );

  return { waveplates, waveplatesTime };
}
