import { useQuery } from "@tanstack/solid-query";
import { createMemo } from "solid-js";
import { ZzzResource } from "@/modules/games/games.constants";
import { createFormattedTime } from "@/modules/games/games.primitives";
import { GameId } from "@/modules/games/games.types";
import { selectResource } from "@/modules/games/games.utils";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import { isStaminaResource } from "@/modules/resources/resources.types";

/** Reactive Zenless Zone Zero resource selectors over the resources query. */
export function createZzzResources() {
  const resourcesQuery = useQuery(() => resourcesQueryOptions());

  const battery = createMemo(() =>
    selectResource(
      resourcesQuery.data,
      GameId.ZenlessZoneZero,
      ZzzResource.Battery,
      isStaminaResource,
    ),
  );
  const batteryTime = createFormattedTime(() => battery()?.fullAt);

  return { battery, batteryTime };
}
