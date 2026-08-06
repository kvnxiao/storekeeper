import { useQuery } from "@tanstack/solid-query";
import { Show, type VoidComponent } from "solid-js";
import { DailyClaimBadge } from "@/modules/daily-rewards/components/DailyClaimBadge";
import { getResourceDisplayName, getResourceIconPath } from "@/modules/games/games.constants";
import { createStaminaResource } from "@/modules/games/games.primitives";
import { GAME_REGISTRY } from "@/modules/games/games.registry";
import type { GameId, GameResourceTypeMap } from "@/modules/games/games.types";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import { GameSection } from "@/modules/ui/components/GameSection";

/** Distributed over `GameId` so a game can only be paired with its own resource. */
export type StaminaGameSectionProps = {
  [G in GameId]: { gameId: G; resourceType: GameResourceTypeMap[G] };
}[GameId];

/**
 * Dashboard section for a game whose only tracked resource is stamina.
 *
 * Both props are read once at setup: the dashboard renders one component per
 * game, so an instance never changes which game it is showing.
 */
export const StaminaGameSection: VoidComponent<StaminaGameSectionProps> = (props) => {
  const query = useQuery(() => resourcesQueryOptions());
  const [stamina, staminaTime] = createStaminaResource(
    () => query.data,
    props.gameId,
    props.resourceType,
  );
  const game = GAME_REGISTRY[props.gameId];

  return (
    <GameSection
      sectionId={props.gameId}
      title={game.name()}
      badge={
        <Show when={game.supportsDailyRewards}>
          <DailyClaimBadge gameId={props.gameId} gameName={game.name()} />
        </Show>
      }
    >
      <StaminaCard
        iconPath={getResourceIconPath(props.resourceType)}
        name={getResourceDisplayName(props.resourceType)}
        data={stamina()}
        formattedTime={staminaTime()}
      />
    </GameSection>
  );
};
