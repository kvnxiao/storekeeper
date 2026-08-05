import { type Accessor, createMemo } from "solid-js";
import { core } from "@/modules/core/core.state";
import type { GameId, GameResourceTypeMap } from "@/modules/games/games.types";
import {
  fillStaminaWhenDue,
  readyCooldownWhenDue,
  selectResource,
} from "@/modules/games/games.utils";
import {
  type AllResources,
  type FormattedTime,
  isCooldownResource,
  isStaminaResource,
} from "@/modules/resources/resources.types";
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

/** Selects a game's stamina resource and derives its formatted full-at time. */
export function createStaminaResource<G extends GameId>(
  resources: Accessor<AllResources | undefined>,
  gameId: G,
  resourceType: GameResourceTypeMap[G],
) {
  const data = createMemo(() =>
    fillStaminaWhenDue(
      selectResource(resources(), gameId, resourceType, isStaminaResource),
      core.tick(),
    ),
  );
  const time = createFormattedTime(() => data()?.fullAt);
  return [data, time] as const;
}

/** Selects a game's cooldown resource and derives its formatted ready-at time. */
export function createCooldownResource<G extends GameId>(
  resources: Accessor<AllResources | undefined>,
  gameId: G,
  resourceType: GameResourceTypeMap[G],
) {
  const data = createMemo(() =>
    readyCooldownWhenDue(
      selectResource(resources(), gameId, resourceType, isCooldownResource),
      core.tick(),
    ),
  );
  const time = createFormattedTime(() => data()?.readyAt);
  return [data, time] as const;
}
