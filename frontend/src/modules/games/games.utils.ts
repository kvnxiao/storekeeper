import type { GameId, GameResourceTypeMap } from "@/modules/games/games.types";
import type { AllResources, GameResource } from "@/modules/resources/resources.types";

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
  const resource = resources?.games?.[gameId]?.find((r: GameResource) => r.type === resourceType);
  if (!resource || !guard(resource.data)) {
    return null;
  }
  return resource.data;
}
