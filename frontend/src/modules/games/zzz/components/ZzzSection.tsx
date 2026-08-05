import type { VoidComponent } from "solid-js";
import { createClaimStatus } from "@/modules/daily-rewards/daily-rewards.primitives";
import { getResourceDisplayName, ZzzResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { createZzzResources } from "@/modules/games/zzz/zzz.primitives";
import { createIsRefreshing } from "@/modules/resources/resources.primitives";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const ZzzSection: VoidComponent = () => {
  const zzz = createZzzResources();
  const claimStatus = createClaimStatus(GameId.ZenlessZoneZero);
  const isRefreshing = createIsRefreshing();

  return (
    <GameSection
      title={m.game_zzz_name()}
      gameId={GameId.ZenlessZoneZero}
      claimStatus={claimStatus()}
    >
      <StaminaCard
        iconPath="/icons/game/zzz/Item_Battery_Charge.webp"
        name={getResourceDisplayName(ZzzResource.Battery)}
        data={zzz.battery() ?? undefined}
        formattedTime={zzz.batteryTime()}
        isRefreshing={isRefreshing()}
      />
    </GameSection>
  );
};
