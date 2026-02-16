import { WaveplatesCard } from "@/modules/games/wuwa/components/WaveplatesCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const WuwaSection: React.FC = () => (
  <GameSection title={m.game_wuthering_waves()}>
    <WaveplatesCard />
  </GameSection>
);
