import { useAtomValue } from "jotai";
import { atoms } from "@/modules/atoms";
import {
  getResourceDisplayName,
  HsrResource,
} from "@/modules/games/games.constants";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { isStaminaResource } from "@/modules/resources/resources.types";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const HsrSection: React.FC = () => {
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);

  const tbpResource = useAtomValue(atoms.games.hsr.trailblazePower);
  const tbpTime = useAtomValue(atoms.games.hsr.trailblazePowerTime);
  const tbpData =
    tbpResource && isStaminaResource(tbpResource.data)
      ? tbpResource.data
      : undefined;

  return (
    <GameSection title={m.game_hsr_name()}>
      <StaminaCard
        iconPath="/icons/game/hsr/Item_Trailblaze_Power.webp"
        name={getResourceDisplayName(HsrResource.TrailblazePower)}
        data={tbpData}
        formattedTime={tbpTime}
        isRefreshing={isRefreshing}
      />
    </GameSection>
  );
};
