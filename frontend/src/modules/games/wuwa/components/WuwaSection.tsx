import type { VoidComponent } from "solid-js";
import { getResourceDisplayName, WuwaResource } from "@/modules/games/games.constants";
import { createWuwaResources } from "@/modules/games/wuwa/wuwa.primitives";
import { createIsRefreshing } from "@/modules/resources/resources.primitives";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const WuwaSection: VoidComponent = () => {
  const wuwa = createWuwaResources();
  const isRefreshing = createIsRefreshing();

  return (
    <GameSection title={m.game_wuwa_name()}>
      <StaminaCard
        iconPath="/icons/game/wuwa/Item_Waveplate.webp"
        name={getResourceDisplayName(WuwaResource.Waveplates)}
        data={wuwa.waveplates() ?? undefined}
        formattedTime={wuwa.waveplatesTime()}
        isRefreshing={isRefreshing()}
      />
    </GameSection>
  );
};
