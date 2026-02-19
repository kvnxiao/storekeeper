import { useAtomValue } from "jotai";
import { atoms } from "@/modules/atoms";
import {
  getResourceDisplayName,
  ZzzResource,
} from "@/modules/games/games.constants";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const ZzzSection: React.FC = () => {
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);

  const batteryData = useAtomValue(atoms.games.zzz.battery);
  const batteryTime = useAtomValue(atoms.games.zzz.batteryTime);

  return (
    <GameSection title={m.game_zzz_name()}>
      <StaminaCard
        iconPath="/icons/game/zzz/Item_Battery_Charge.webp"
        name={getResourceDisplayName(ZzzResource.Battery)}
        data={batteryData ?? undefined}
        formattedTime={batteryTime}
        isRefreshing={isRefreshing}
      />
    </GameSection>
  );
};
