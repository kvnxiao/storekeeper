import type { VoidComponent } from "solid-js";
import { DailyClaimBadge } from "@/modules/daily-rewards/components/DailyClaimBadge";
import {
  getResourceDisplayName,
  getResourceIconPath,
  ZzzResource,
} from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { createZzzResources } from "@/modules/games/zzz/zzz.primitives";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const ZzzSection: VoidComponent = () => {
  const zzz = createZzzResources();

  return (
    <GameSection
      sectionId={GameId.ZenlessZoneZero}
      title={m.game_zzz_name()}
      badge={<DailyClaimBadge gameId={GameId.ZenlessZoneZero} />}
    >
      <StaminaCard
        iconPath={getResourceIconPath(ZzzResource.Battery)}
        name={getResourceDisplayName(ZzzResource.Battery)}
        data={zzz.battery()}
        formattedTime={zzz.batteryTime()}
      />
    </GameSection>
  );
};
