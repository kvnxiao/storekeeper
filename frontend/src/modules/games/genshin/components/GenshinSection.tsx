import type { VoidComponent } from "solid-js";
import { createClaimStatus } from "@/modules/daily-rewards/daily-rewards.primitives";
import {
  GenshinResource,
  getResourceDisplayName,
  getResourceIconPath,
} from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { ExpeditionsCard } from "@/modules/games/genshin/components/ExpeditionsCard";
import { createGenshinResources } from "@/modules/games/genshin/genshin.primitives";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { createIsRefreshing } from "@/modules/resources/resources.primitives";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const GenshinSection: VoidComponent = () => {
  const genshin = createGenshinResources();
  const claimStatus = createClaimStatus(GameId.GenshinImpact);
  const isRefreshing = createIsRefreshing();

  return (
    <GameSection
      title={m.game_genshin_name()}
      gameId={GameId.GenshinImpact}
      claimStatus={claimStatus()}
    >
      <StaminaCard
        iconPath={getResourceIconPath(GenshinResource.Resin)}
        name={getResourceDisplayName(GenshinResource.Resin)}
        data={genshin.resin() ?? undefined}
        formattedTime={genshin.resinTime()}
        isRefreshing={isRefreshing()}
      />
      <CooldownCard
        iconPath={getResourceIconPath(GenshinResource.ParametricTransformer)}
        name={getResourceDisplayName(GenshinResource.ParametricTransformer)}
        data={genshin.parametricTransformer() ?? undefined}
        formattedTime={genshin.parametricTransformerTime()}
        isRefreshing={isRefreshing()}
      />
      <StaminaCard
        iconPath={getResourceIconPath(GenshinResource.RealmCurrency)}
        name={getResourceDisplayName(GenshinResource.RealmCurrency)}
        data={genshin.realmCurrency() ?? undefined}
        formattedTime={genshin.realmCurrencyTime()}
        isRefreshing={isRefreshing()}
      />
      <ExpeditionsCard
        expeditions={genshin.expeditions()}
        ready={genshin.expeditionsReady()}
        formattedTime={genshin.expeditionsTime()}
        isRefreshing={isRefreshing()}
      />
    </GameSection>
  );
};
