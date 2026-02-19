import { useAtomValue } from "jotai";
import { atoms } from "@/modules/atoms";
import {
  getResourceDisplayName,
  HsrResource,
} from "@/modules/games/games.constants";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const HsrSection: React.FC = () => {
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);

  const tbpData = useAtomValue(atoms.games.hsr.trailblazePower);
  const tbpTime = useAtomValue(atoms.games.hsr.trailblazePowerTime);

  return (
    <GameSection title={m.game_hsr_name()}>
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
