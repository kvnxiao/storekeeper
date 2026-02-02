import { useAtomValue } from "jotai";
import { useMemo } from "react";

import { atoms } from "@/modules/atoms";
import { ResourceIcon } from "@/modules/resources/components/ResourceIcon";
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
  const tick = useAtomValue(atoms.core.tick);

  const name = getResourceDisplayName(type);
  const percentage = (data.current / data.max) * 100;
  const isFull = data.current >= data.max;
  const progressColor = getProgressColor(percentage, isFull);

  const timeRemaining = useMemo(
    () => formatTimeRemaining(data.fullAt, tick),
    [data.fullAt, tick],
  );

  return (
    <div className="rounded-lg bg-zinc-50 p-2 dark:bg-zinc-700">
      <div className="flex items-center gap-2">
        <ResourceIcon type={type} size="md" />
        <div className="flex min-w-0 flex-1 items-baseline justify-between gap-2">
          <span className="truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
            {name}
          </span>
          <span className="shrink-0 text-sm tabular-nums text-zinc-950 dark:text-white">
            <span className="font-semibold">{data.current}</span>
            <span className="text-zinc-500 dark:text-zinc-400">
              /{data.max}
            </span>
          </span>
        </div>
      </div>
      <ProgressBar
        value={Math.min(percentage, 100)}
        minValue={0}
        maxValue={100}
        color={progressColor}
        size="xs"
        className="mt-1.5"
        aria-label={`${name} progress`}
      />
      <div className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
        {isFull ? "Full!" : `Full in ${timeRemaining}`}
      </div>
    </div>
  );
};
