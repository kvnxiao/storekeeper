import type { VoidComponent } from "solid-js";
import { DailyClaimBadge } from "@/modules/daily-rewards/components/DailyClaimBadge";
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
  const isRefreshing = createIsRefreshing();

  return (
    <GameSection
      title={m.game_hsr_name()}
      badge={<DailyClaimBadge gameId={GameId.HonkaiStarRail} />}
    >
      <StaminaCard
        iconPath={getResourceIconPath(HsrResource.TrailblazePower)}
        name={getResourceDisplayName(HsrResource.TrailblazePower)}
        data={hsr.trailblazePower()}
        formattedTime={hsr.trailblazePowerTime()}
        isRefreshing={isRefreshing()}
      />
    </GameSection>
  );
};
