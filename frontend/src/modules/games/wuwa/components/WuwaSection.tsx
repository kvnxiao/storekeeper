import { WaveplatesCard } from "@/modules/games/wuwa/components/WaveplatesCard";
import { GameSectionWrapper } from "@/modules/ui/components/GameSectionWrapper";

export const WuwaSection: React.FC = () => (
  <GameSectionWrapper title="Wuthering Waves">
    <WaveplatesCard />
  </GameSectionWrapper>
);
