import { atoms } from "@/modules/atoms";
import { GenshinResource } from "@/modules/games/games.constants";
import { ExpeditionsCard } from "@/modules/games/genshin/components/ExpeditionsCard";
import { ResourceCard } from "@/modules/resources/components/ResourceCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const GenshinSection: React.FC = () => (
  <GameSection title={m.game_genshin_impact()}>
    <ResourceCard
      resourceAtom={atoms.games.genshin.resin}
      resourceType={GenshinResource.Resin}
      iconPath="/icons/game/genshin/Item_Original_Resin.webp"
      variant="stamina"
    />
    <ResourceCard
      resourceAtom={atoms.games.genshin.parametricTransformer}
      resourceType={GenshinResource.ParametricTransformer}
      iconPath="/icons/game/genshin/Item_Parametric_Transformer.webp"
      variant="cooldown"
    />
    <ResourceCard
      resourceAtom={atoms.games.genshin.realmCurrency}
      resourceType={GenshinResource.RealmCurrency}
      iconPath="/icons/game/genshin/Item_Realm_Currency.webp"
      variant="stamina"
    />
    <ExpeditionsCard />
  </GameSection>
);
