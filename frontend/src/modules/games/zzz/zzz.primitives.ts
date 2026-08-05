import { useQuery } from "@tanstack/solid-query";
import { ZzzResource } from "@/modules/games/games.constants";
import { createStaminaResource } from "@/modules/games/games.primitives";
import { GameId } from "@/modules/games/games.types";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";

/** Reactive Zenless Zone Zero resource selectors over the resources query. */
export function createZzzResources() {
  const query = useQuery(() => resourcesQueryOptions());
  const [battery, batteryTime] = createStaminaResource(
    () => query.data,
    GameId.ZenlessZoneZero,
    ZzzResource.Battery,
  );

  return { battery, batteryTime };
}
