import { ExpeditionsCard } from "@/modules/games/genshin/components/ExpeditionsCard";
import { ParametricTransformerCard } from "@/modules/games/genshin/components/ParametricTransformerCard";
import { RealmCurrencyCard } from "@/modules/games/genshin/components/RealmCurrencyCard";
import { ResinCard } from "@/modules/games/genshin/components/ResinCard";
import { GameSection } from "@/modules/ui/components/GameSection";

export const GenshinSection: React.FC = () => (
  <GameSection title="Genshin Impact">
    <ResinCard />
    <ParametricTransformerCard />
    <RealmCurrencyCard />
    <ExpeditionsCard />
  </GameSection>
);
