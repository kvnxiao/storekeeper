import { useAtomValue } from "jotai";
import { atoms } from "@/modules/atoms";
import {
  GenshinResource,
  getResourceDisplayName,
} from "@/modules/games/games.constants";
import { ExpeditionsCard } from "@/modules/games/genshin/components/ExpeditionsCard";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const GenshinSection: React.FC = () => {
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);

  const resinData = useAtomValue(atoms.games.genshin.resin);
  const resinTime = useAtomValue(atoms.games.genshin.resinTime);

  const ptData = useAtomValue(atoms.games.genshin.parametricTransformer);
  const ptTime = useAtomValue(atoms.games.genshin.parametricTransformerTime);

  const realmData = useAtomValue(atoms.games.genshin.realmCurrency);
  const realmTime = useAtomValue(atoms.games.genshin.realmCurrencyTime);

  return (
    <GameSection title={m.game_genshin_name()}>
      <StaminaCard
        iconPath="/icons/game/genshin/Item_Original_Resin.webp"
        name={getResourceDisplayName(GenshinResource.Resin)}
        data={resinData ?? undefined}
        formattedTime={resinTime}
        isRefreshing={isRefreshing}
      />
      <CooldownCard
        iconPath="/icons/game/genshin/Item_Parametric_Transformer.webp"
        name={getResourceDisplayName(GenshinResource.ParametricTransformer)}
        data={ptData ?? undefined}
        formattedTime={ptTime}
        isRefreshing={isRefreshing}
      />
      <StaminaCard
        iconPath="/icons/game/genshin/Item_Realm_Currency.webp"
        name={getResourceDisplayName(GenshinResource.RealmCurrency)}
        data={realmData ?? undefined}
        formattedTime={realmTime}
        isRefreshing={isRefreshing}
      />
      <ExpeditionsCard />
    </GameSection>
  );
};
