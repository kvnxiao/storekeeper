import { useQuery } from "@tanstack/solid-query";
import { createMemo } from "solid-js";
import { WuwaResource } from "@/modules/games/games.constants";
import { createFormattedTime } from "@/modules/games/games.primitives";
import { GameId } from "@/modules/games/games.types";
import { selectResource } from "@/modules/games/games.utils";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import { isStaminaResource } from "@/modules/resources/resources.types";

/** Reactive Wuthering Waves resource selectors over the resources query. */
export function createWuwaResources() {
  const resourcesQuery = useQuery(() => resourcesQueryOptions());

  const waveplates = createMemo(() =>
    selectResource(
      resourcesQuery.data,
      GameId.WutheringWaves,
      WuwaResource.Waveplates,
      isStaminaResource,
    ),
  );
  const waveplatesTime = createFormattedTime(() => waveplates()?.fullAt);

  return { waveplates, waveplatesTime };
}
