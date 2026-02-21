import { useAtomValue } from "jotai";
import { atoms } from "@/modules/atoms";
import {
  getResourceDisplayName,
  HsrResource,
} from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const HsrSection: React.FC = () => {
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);
  const claimStatusMap = useAtomValue(atoms.core.dailyClaimStatus);
  const claimStatus = claimStatusMap.get(GameId.HonkaiStarRail) ?? null;

  const tbpData = useAtomValue(atoms.games.hsr.trailblazePower);
  const tbpTime = useAtomValue(atoms.games.hsr.trailblazePowerTime);

  return (
    <GameSection title={m.game_hsr_name()} claimStatus={claimStatus}>
      <StaminaCard
        iconPath="/icons/game/hsr/Item_Trailblaze_Power.webp"
        name={getResourceDisplayName(HsrResource.TrailblazePower)}
        data={tbpData ?? undefined}
        formattedTime={tbpTime}
        isRefreshing={isRefreshing}
      />
    </GameSection>
  );
};
