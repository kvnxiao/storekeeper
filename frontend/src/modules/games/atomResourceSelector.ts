import { atom } from "jotai";
import type { CoreAtoms } from "@/modules/core/core.atoms";
import type { GameId, GameResourceTypeMap } from "@/modules/games/games.types";
import type { GameResource } from "@/modules/resources/resources.types";

/**
 * Creates a derived atom that selects a single resource by game and type,
 * narrowing `.data` via the provided type guard.
 *
 * Returns the narrowed data (`T`) or `null` if not found / guard fails.
 */
export function atomResourceSelector<G extends GameId, T>(
  getCore: () => CoreAtoms,
  gameId: G,
  resourceType: GameResourceTypeMap[G],
  guard: (data: unknown) => data is T,
) {
  return atom<T | null>((get) => {
    const { data } = get(getCore().resourcesQuery);
    const resource = data?.games?.[gameId]?.find(
      (r: GameResource) => r.type === resourceType,
    );
    if (!resource || !guard(resource.data)) return null;
    return resource.data;
  });
}
