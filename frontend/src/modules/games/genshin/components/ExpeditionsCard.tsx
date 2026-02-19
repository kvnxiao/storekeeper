import { useAtomValue } from "jotai";
import { atoms } from "@/modules/atoms";
import {
  GenshinResource,
  getResourceDisplayName,
} from "@/modules/games/games.constants";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import { isExpeditionResource } from "@/modules/resources/resources.types";

const RESOURCE_ICON = "/icons/game/genshin/Expeditions.webp";

export const ExpeditionsCard: React.FC = () => {
  const resource = useAtomValue(atoms.games.genshin.expeditions);
  const allDone = useAtomValue(atoms.games.genshin.expeditionsReady);
  const expeditionsTime = useAtomValue(atoms.games.genshin.expeditionsTime);
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);
  const resourceName = getResourceDisplayName(GenshinResource.Expeditions);

  const data =
    resource && isExpeditionResource(resource.data)
      ? { isReady: allDone, readyAt: resource.data.earliestFinishAt }
      : undefined;

  return (
    <CooldownCard
      iconPath={RESOURCE_ICON}
      name={resourceName}
      data={data}
      formattedTime={expeditionsTime}
      isRefreshing={isRefreshing}
    />
  );
};
