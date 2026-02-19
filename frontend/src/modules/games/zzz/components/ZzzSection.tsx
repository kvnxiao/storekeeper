import { useAtomValue } from "jotai";
import { atoms } from "@/modules/atoms";
import {
  getResourceDisplayName,
  ZzzResource,
} from "@/modules/games/games.constants";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { isStaminaResource } from "@/modules/resources/resources.types";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const ZzzSection: React.FC = () => {
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);

  const batteryResource = useAtomValue(atoms.games.zzz.battery);
  const batteryTime = useAtomValue(atoms.games.zzz.batteryTime);
  const batteryData =
    batteryResource && isStaminaResource(batteryResource.data)
      ? batteryResource.data
      : undefined;

  return (
    <GameSection title={m.game_zzz_name()}>
      <StaminaCard
        iconPath="/icons/game/zzz/Item_Battery_Charge.webp"
        name={getResourceDisplayName(ZzzResource.Battery)}
        data={batteryData}
        formattedTime={batteryTime}
        isRefreshing={isRefreshing}
      />
    </GameSection>
  );
};
