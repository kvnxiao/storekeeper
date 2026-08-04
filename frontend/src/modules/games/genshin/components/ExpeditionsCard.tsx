import type { VoidComponent } from "solid-js";
import { core } from "@/modules/core/core.state";
import { GenshinResource, getResourceDisplayName } from "@/modules/games/games.constants";
import { createGenshinResources } from "@/modules/games/genshin/genshin.primitives";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";

const RESOURCE_ICON = "/icons/game/genshin/Expeditions.webp";

export const ExpeditionsCard: VoidComponent = () => {
  const genshin = createGenshinResources();

  const data = () => {
    const expeditions = genshin.expeditions();
    return expeditions
      ? { isReady: genshin.expeditionsReady(), readyAt: expeditions.earliestFinishAt }
      : undefined;
  };

  return (
    <CooldownCard
      iconPath={RESOURCE_ICON}
      name={getResourceDisplayName(GenshinResource.Expeditions)}
      data={data()}
      formattedTime={genshin.expeditionsTime()}
      isRefreshing={core.isRefreshing()}
    />
  );
};
