import { BatteryCard } from "@/modules/games/zzz/components/BatteryCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const ZzzSection: React.FC = () => (
  <GameSection title={m.game_zenless_zone_zero()}>
    <BatteryCard />
  </GameSection>
);
