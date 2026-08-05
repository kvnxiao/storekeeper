import type { VoidComponent } from "solid-js";
import {
  GenshinResource,
  getResourceDisplayName,
  getResourceIconPath,
} from "@/modules/games/games.constants";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import type { ExpeditionResource, FormattedTime } from "@/modules/resources/resources.types";

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
      iconPath={getResourceIconPath(GenshinResource.Expeditions)}
      name={getResourceDisplayName(GenshinResource.Expeditions)}
      data={data()}
      formattedTime={props.formattedTime}
      isRefreshing={props.isRefreshing}
    />
  );
};
