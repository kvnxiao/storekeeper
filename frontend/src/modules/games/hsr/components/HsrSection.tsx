import type { VoidComponent } from "solid-js";
import { createClaimStatus } from "@/modules/daily-rewards/daily-rewards.primitives";
import {
  getResourceDisplayName,
  getResourceIconPath,
  HsrResource,
} from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { createHsrResources } from "@/modules/games/hsr/hsr.primitives";
import { createIsRefreshing } from "@/modules/resources/resources.primitives";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const HsrSection: VoidComponent = () => {
  const hsr = createHsrResources();
  const claimStatus = createClaimStatus(GameId.HonkaiStarRail);
  const isRefreshing = createIsRefreshing();

  return (
    <GameSection
      title={m.game_hsr_name()}
      gameId={GameId.HonkaiStarRail}
      claimStatus={claimStatus()}
    >
      <StaminaCard
        iconPath={getResourceIconPath(HsrResource.TrailblazePower)}
        name={getResourceDisplayName(HsrResource.TrailblazePower)}
        data={hsr.trailblazePower() ?? undefined}
        formattedTime={hsr.trailblazePowerTime()}
        isRefreshing={isRefreshing()}
      />
    </GameSection>
  );
};
