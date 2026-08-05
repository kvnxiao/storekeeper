import type { VoidComponent } from "solid-js";
import { DailyClaimBadge } from "@/modules/daily-rewards/components/DailyClaimBadge";
import {
  GenshinResource,
  getResourceDisplayName,
  getResourceIconPath,
} from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { createGenshinResources } from "@/modules/games/genshin/genshin.primitives";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const GenshinSection: VoidComponent = () => {
  const genshin = createGenshinResources();

  return (
    <GameSection
      sectionId={GameId.GenshinImpact}
      title={m.game_genshin_name()}
      badge={<DailyClaimBadge gameId={GameId.GenshinImpact} />}
    >
      <StaminaCard
        iconPath={getResourceIconPath(GenshinResource.Resin)}
        name={getResourceDisplayName(GenshinResource.Resin)}
        data={genshin.resin()}
        formattedTime={genshin.resinTime()}
      />
      <CooldownCard
        iconPath={getResourceIconPath(GenshinResource.ParametricTransformer)}
        name={getResourceDisplayName(GenshinResource.ParametricTransformer)}
        data={genshin.parametricTransformer()}
        formattedTime={genshin.parametricTransformerTime()}
      />
      <StaminaCard
        iconPath={getResourceIconPath(GenshinResource.RealmCurrency)}
        name={getResourceDisplayName(GenshinResource.RealmCurrency)}
        data={genshin.realmCurrency()}
        formattedTime={genshin.realmCurrencyTime()}
      />
      <CooldownCard
        iconPath={getResourceIconPath(GenshinResource.Expeditions)}
        name={getResourceDisplayName(GenshinResource.Expeditions)}
        data={genshin.expeditionsCooldown()}
        formattedTime={genshin.expeditionsTime()}
      />
    </GameSection>
  );
};
