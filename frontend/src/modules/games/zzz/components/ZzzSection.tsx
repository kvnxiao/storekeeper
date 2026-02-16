import { atoms } from "@/modules/atoms";
import { ZzzResource } from "@/modules/games/games.constants";
import { ResourceCard } from "@/modules/resources/components/ResourceCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const ZzzSection: React.FC = () => (
  <GameSection title={m.game_zenless_zone_zero()}>
    <ResourceCard
      resourceAtom={atoms.games.zzz.battery}
      resourceType={ZzzResource.Battery}
      iconPath="/icons/game/zzz/Item_Battery_Charge.webp"
      variant="stamina"
    />
  </GameSection>
);
