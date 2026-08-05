import { type Accessor, createMemo } from "solid-js";
import { core } from "@/modules/core/core.state";
import type { GameId, GameResourceTypeMap } from "@/modules/games/games.types";
import { selectResource } from "@/modules/games/games.utils";
import {
  type AllResources,
  type FormattedTime,
  isCooldownResource,
  isStaminaResource,
} from "@/modules/resources/resources.types";
import {
  formatAbsoluteDateTime,
  formatTimeRemaining,
  isPastDateTime,
} from "@/modules/resources/resources.utils";

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
 * Selects a game's stamina resource and derives its formatted full-at time.
 * Clamps `current` to `max` once `fullAt` passes the tick, so fullness stays
 * consistent with the tick-driven countdown between backend polls.
 */
export function createStaminaResource<G extends GameId>(
  resources: Accessor<AllResources | undefined>,
  gameId: G,
  resourceType: GameResourceTypeMap[G],
) {
  const data = createMemo(() => {
    const resource = selectResource(resources(), gameId, resourceType, isStaminaResource);
    if (!resource || resource.current >= resource.max) {
      return resource;
    }
    return isPastDateTime(resource.fullAt, core.tick())
      ? { ...resource, current: resource.max }
      : resource;
  });
  const time = createFormattedTime(() => data()?.fullAt);
  return [data, time] as const;
}

/**
 * Selects a game's cooldown resource and derives its formatted ready-at time.
 * Marks it ready once `readyAt` passes the tick, so readiness stays
 * consistent with the tick-driven countdown between backend polls.
 */
export function createCooldownResource<G extends GameId>(
  resources: Accessor<AllResources | undefined>,
  gameId: G,
  resourceType: GameResourceTypeMap[G],
) {
  const data = createMemo(() => {
    const resource = selectResource(resources(), gameId, resourceType, isCooldownResource);
    if (!resource || resource.isReady) {
      return resource;
    }
    return isPastDateTime(resource.readyAt, core.tick())
      ? { ...resource, isReady: true }
      : resource;
  });
  const time = createFormattedTime(() => data()?.readyAt);
  return [data, time] as const;
}
