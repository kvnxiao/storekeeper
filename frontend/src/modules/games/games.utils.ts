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
 * Fills a stamina resource once its `fullAt` has passed, so fullness stays
 * consistent with the countdown the UI derives from the same timestamp between
 * backend polls.
 */
export function fillStaminaWhenDue(
  resource: StaminaResource | null,
  now: Temporal.Instant,
): StaminaResource | null {
  if (!resource || resource.current >= resource.max) {
    return resource;
  }
  return isPastDateTime(resource.fullAt, now) ? { ...resource, current: resource.max } : resource;
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
