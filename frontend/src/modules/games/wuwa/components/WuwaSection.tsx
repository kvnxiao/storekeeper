import { useAtomValue } from "jotai";
import { atoms } from "@/modules/atoms";
import {
  getResourceDisplayName,
  WuwaResource,
} from "@/modules/games/games.constants";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const WuwaSection: React.FC = () => {
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);

  const waveplatesData = useAtomValue(atoms.games.wuwa.waveplates);
  const waveplatesTime = useAtomValue(atoms.games.wuwa.waveplatesTime);

  return (
    <GameSection title={m.game_wuwa_name()}>
      <StaminaCard
        iconPath="/icons/game/wuwa/Item_Waveplate.webp"
        name={getResourceDisplayName(WuwaResource.Waveplates)}
        data={waveplatesData ?? undefined}
        formattedTime={waveplatesTime}
        isRefreshing={isRefreshing}
      />
    </GameSection>
  );
};
