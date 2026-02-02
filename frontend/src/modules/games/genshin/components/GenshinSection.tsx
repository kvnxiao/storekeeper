import { ExpeditionsCard } from "@/modules/games/genshin/components/ExpeditionsCard";
import { ParametricTransformerCard } from "@/modules/games/genshin/components/ParametricTransformerCard";
import { RealmCurrencyCard } from "@/modules/games/genshin/components/RealmCurrencyCard";
import { ResinCard } from "@/modules/games/genshin/components/ResinCard";
import { GameSectionWrapper } from "@/modules/ui/components/GameSectionWrapper";

export const GenshinSection: React.FC = () => (
  <GameSectionWrapper title="Genshin Impact">
    <ResinCard />
    <ParametricTransformerCard />
    <RealmCurrencyCard />
    <ExpeditionsCard />
  </GameSectionWrapper>
);
