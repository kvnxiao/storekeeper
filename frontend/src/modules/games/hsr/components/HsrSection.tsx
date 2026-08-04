import type { VoidComponent } from "solid-js";
import { core } from "@/modules/core/core.state";
import { getResourceDisplayName, HsrResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { createHsrResources } from "@/modules/games/hsr/hsr.primitives";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const HsrSection: VoidComponent = () => {
  const hsr = createHsrResources();

  return (
    <GameSection
      title={m.game_hsr_name()}
      gameId={GameId.HonkaiStarRail}
      claimStatus={core.dailyClaimStatus().get(GameId.HonkaiStarRail) ?? null}
    >
      <StaminaCard
        iconPath="/icons/game/hsr/Item_Trailblaze_Power.webp"
        name={getResourceDisplayName(HsrResource.TrailblazePower)}
        data={hsr.trailblazePower() ?? undefined}
        formattedTime={hsr.trailblazePowerTime()}
        isRefreshing={core.isRefreshing()}
      />
    </GameSection>
  );
};
