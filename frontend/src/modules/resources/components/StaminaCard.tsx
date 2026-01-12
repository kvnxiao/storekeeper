import { useAtomValue } from "jotai";
import { useMemo } from "react";

import { currentTick } from "@/modules/core/core.tick";
import type { StaminaResource } from "@/modules/resources/resources.types";
import {
  formatTimeRemaining,
  getProgressColor,
  getResourceDisplayName,
} from "@/modules/resources/resources.utils";
import { ProgressBar } from "@/modules/ui/components/ProgressBar";

interface StaminaCardProps {
  type: string;
  data: StaminaResource;
}

export const StaminaCard: React.FC<StaminaCardProps> = ({ type, data }) => {
  const tick = useAtomValue(currentTick);

  const name = getResourceDisplayName(type);
  const percentage = (data.current / data.max) * 100;
  const isFull = data.current >= data.max;
  const progressColor = getProgressColor(percentage, isFull);

  const timeRemaining = useMemo(
    () => formatTimeRemaining(data.fullAt, tick),
    [data.fullAt, tick],
  );

  return (
    <div className="rounded-lg bg-zinc-50 p-3 dark:bg-zinc-800/50">
      <div className="mb-1 text-xs font-medium text-zinc-500 dark:text-zinc-400">
        {name}
      </div>
      <div className="text-xl font-bold text-zinc-950 dark:text-white">
        {data.current}{" "}
        <span className="text-sm font-normal text-zinc-500 dark:text-zinc-400">
          / {data.max}
        </span>
      </div>
      <ProgressBar
        value={Math.min(percentage, 100)}
        minValue={0}
        maxValue={100}
        color={progressColor}
        className="mt-2"
        aria-label={`${name} progress`}
      />
      <div className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
        {isFull ? "Full!" : `Full in ${timeRemaining}`}
      </div>
    </div>
  );
};
