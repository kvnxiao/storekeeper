import { useQuery } from "@tanstack/solid-query";
import { type Accessor, createMemo } from "solid-js";
import { core } from "@/modules/core/core.state";
import type { GameId, GameResourceTypeMap } from "@/modules/games/games.types";
import { selectResource } from "@/modules/games/games.utils";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import { type FormattedTime, isStaminaResource } from "@/modules/resources/resources.types";
import { formatAbsoluteDateTime, formatTimeRemaining } from "@/modules/resources/resources.utils";

/**
 * Derives formatted time for a resource datetime.
 *
 * Re-evaluates on tick, locale change, or when the source datetime changes.
 */
export function createFormattedTime(
  datetime: Accessor<string | null | undefined>,
): Accessor<FormattedTime> {
  return createMemo(() => ({
    relativeTime: formatTimeRemaining(datetime(), core.tick(), core.durationFormatter()),
    absoluteTime: formatAbsoluteDateTime(
      datetime(),
      core.tick(),
      core.timeOnlyFormatter(),
      core.weekdayTimeFormatter(),
    ),
  }));
}

/**
 * Selects a game's stamina resource from the resources query and derives its
 * formatted full-at time.
 */
export function createStaminaResource<G extends GameId>(
  gameId: G,
  resourceType: GameResourceTypeMap[G],
) {
  const query = useQuery(() => resourcesQueryOptions());
  const data = createMemo(() =>
    selectResource(query.data, gameId, resourceType, isStaminaResource),
  );
  const time = createFormattedTime(() => data()?.fullAt);
  return [data, time] as const;
}
