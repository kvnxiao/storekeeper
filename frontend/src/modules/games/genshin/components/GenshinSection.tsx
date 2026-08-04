import type { VoidComponent } from "solid-js";
import { core } from "@/modules/core/core.state";
import { GenshinResource, getResourceDisplayName } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { ExpeditionsCard } from "@/modules/games/genshin/components/ExpeditionsCard";
import { createGenshinResources } from "@/modules/games/genshin/genshin.primitives";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const GenshinSection: VoidComponent = () => {
  const genshin = createGenshinResources();

  return (
    <GameSection
      title={m.game_genshin_name()}
      gameId={GameId.GenshinImpact}
      claimStatus={core.dailyClaimStatus().get(GameId.GenshinImpact) ?? null}
    >
      <StaminaCard
        iconPath="/icons/game/genshin/Item_Original_Resin.webp"
        name={getResourceDisplayName(GenshinResource.Resin)}
        data={genshin.resin() ?? undefined}
        formattedTime={genshin.resinTime()}
        isRefreshing={core.isRefreshing()}
      />
      <CooldownCard
        iconPath="/icons/game/genshin/Item_Parametric_Transformer.webp"
        name={getResourceDisplayName(GenshinResource.ParametricTransformer)}
        data={genshin.parametricTransformer() ?? undefined}
        formattedTime={genshin.parametricTransformerTime()}
        isRefreshing={core.isRefreshing()}
      />
      <StaminaCard
        iconPath="/icons/game/genshin/Item_Realm_Currency.webp"
        name={getResourceDisplayName(GenshinResource.RealmCurrency)}
        data={genshin.realmCurrency() ?? undefined}
        formattedTime={genshin.realmCurrencyTime()}
        isRefreshing={core.isRefreshing()}
      />
      <ExpeditionsCard />
    </GameSection>
  );
};
