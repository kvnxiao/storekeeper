import { atoms } from "@/modules/atoms";
import { HsrResource } from "@/modules/games/games.constants";
import { ResourceCard } from "@/modules/resources/components/ResourceCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const HsrSection: React.FC = () => (
  <GameSection title={m.game_hsr_name()}>
    <ResourceCard
      resourceAtom={atoms.games.hsr.trailblazePower}
      resourceType={HsrResource.TrailblazePower}
      iconPath="/icons/game/hsr/Item_Trailblaze_Power.webp"
      variant="stamina"
    />
  </GameSection>
);
