import { parseInstant } from "@/modules/core/core.utils";
import type { GameId, GameResourceTypeMap } from "@/modules/games/games.types";
import type {
  AllResources,
  CooldownResource,
  StaminaResource,
} from "@/modules/resources/resources.types";
import { isPastDateTime } from "@/modules/resources/resources.utils";

/**
 * Selects a single resource by game and type, narrowing `.data` via the
 * provided type guard. Returns `null` if not found or the guard fails.
 */
export function selectResource<G extends GameId, T>(
  resources: AllResources | undefined,
  gameId: G,
  resourceType: GameResourceTypeMap[G],
  guard: (data: unknown) => data is T,
): T | null {
  const resource = resources?.games?.[gameId]?.find((r) => r.type === resourceType);
  if (!resource || !guard(resource.data)) {
    return null;
  }
  return resource.data;
}

/**
 * Re-derives a stamina resource's `current` from `fullAt` between backend
 * polls, using the same timestamp as the countdown. While the resource is
 * incomplete, clamps the estimate to the polled value when `max - current` is
 * not divisible by the step size.
 */
export function estimateStaminaCurrent(
  resource: StaminaResource | null,
  now: Temporal.Instant,
): StaminaResource | null {
  if (!resource || resource.current >= resource.max || resource.regenRateSeconds <= 0) {
    return resource;
  }
  const fullAt = parseInstant(resource.fullAt);
  if (fullAt === null || Temporal.Instant.compare(fullAt, now) <= 0) {
    return { ...resource, current: resource.max };
  }
  const secondsToFull = now.until(fullAt, {
    largestUnit: "second",
    smallestUnit: "second",
    roundingMode: "trunc",
  }).seconds;
  const remaining = Math.ceil(secondsToFull / resource.regenRateSeconds) * resource.regenStepUnits;
  return { ...resource, current: Math.max(resource.max - remaining, resource.current) };
}

/** Marks a cooldown resource ready once its `readyAt` has passed. */
export function readyCooldownWhenDue(
  resource: CooldownResource | null,
  now: Temporal.Instant,
): CooldownResource | null {
  if (!resource || resource.isReady) {
    return resource;
  }
  return isPastDateTime(resource.readyAt, now) ? { ...resource, isReady: true } : resource;
}
