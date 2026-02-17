import { atoms } from "@/modules/atoms";
import { WuwaResource } from "@/modules/games/games.constants";
import { ResourceCard } from "@/modules/resources/components/ResourceCard";
import { GameSection } from "@/modules/ui/components/GameSection";
import * as m from "@/paraglide/messages";

export const WuwaSection: React.FC = () => (
  <GameSection title={m.game_wuwa_name()}>
    <ResourceCard
      resourceAtom={atoms.games.wuwa.waveplates}
      resourceType={WuwaResource.Waveplates}
      iconPath="/icons/game/wuwa/Item_Waveplate.webp"
      variant="stamina"
    />
  </GameSection>
);
