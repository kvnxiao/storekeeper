import type { VoidComponent } from "solid-js";
import { GenshinResource, getResourceDisplayName } from "@/modules/games/games.constants";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import type { ExpeditionResource, FormattedTime } from "@/modules/resources/resources.types";

const RESOURCE_ICON = "/icons/game/genshin/Expeditions.webp";

export interface ExpeditionsCardProps {
  expeditions: ExpeditionResource | null;
  ready: boolean;
  formattedTime: FormattedTime;
  isRefreshing: boolean;
}

export const ExpeditionsCard: VoidComponent<ExpeditionsCardProps> = (props) => {
  const data = () => {
    const expeditions = props.expeditions;
    return expeditions
      ? { isReady: props.ready, readyAt: expeditions.earliestFinishAt }
      : undefined;
  };

  return (
    <CooldownCard
      iconPath={RESOURCE_ICON}
      name={getResourceDisplayName(GenshinResource.Expeditions)}
      data={data()}
      formattedTime={props.formattedTime}
      isRefreshing={props.isRefreshing}
    />
  );
};
