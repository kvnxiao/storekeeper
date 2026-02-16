import { atom } from "jotai";
import type { CoreAtoms } from "@/modules/core/core.atoms";
import type { GameId } from "@/modules/games/games.types";
import type { GameResource } from "@/modules/resources/resources.types";

/**
 * Creates a derived atom that selects a single resource by game and type.
 *
 * Reads from the core resources query and returns the matching resource
 * or null if not found.
 */
export function createResourceSelector(
  getCore: () => CoreAtoms,
  gameId: GameId,
  resourceType: string,
) {
  return atom((get) => {
    const { data } = get(getCore().resourcesQuery);
    return (
      (data?.games?.[gameId]?.find(
        (r: GameResource) => r.type === resourceType,
      ) as GameResource | undefined) ?? null
    );
  });
}
