import { TrailblazePowerCard } from "@/modules/games/hsr/components/TrailblazePowerCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const HsrSection: React.FC = () => (
  <GameSection title={m.game_honkai_star_rail()}>
    <TrailblazePowerCard />
  </GameSection>
);
