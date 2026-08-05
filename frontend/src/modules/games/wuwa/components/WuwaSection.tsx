import type { VoidComponent } from "solid-js";
import {
  getResourceDisplayName,
  getResourceIconPath,
  WuwaResource,
} from "@/modules/games/games.constants";
import { createWuwaResources } from "@/modules/games/wuwa/wuwa.primitives";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const WuwaSection: VoidComponent = () => {
  const wuwa = createWuwaResources();

  return (
    <GameSection title={m.game_wuwa_name()}>
      <StaminaCard
        iconPath={getResourceIconPath(WuwaResource.Waveplates)}
        name={getResourceDisplayName(WuwaResource.Waveplates)}
        data={wuwa.waveplates()}
        formattedTime={wuwa.waveplatesTime()}
      />
    </GameSection>
  );
};
