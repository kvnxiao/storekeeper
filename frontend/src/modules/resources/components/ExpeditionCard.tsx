import { useAtomValue } from "jotai";
import { useMemo } from "react";

import { currentTick } from "@/modules/core/core.tick";
import type { ExpeditionResource } from "@/modules/resources/resources.types";
import {
  formatTimeRemaining,
  getResourceDisplayName,
  isPastDateTime,
} from "@/modules/resources/resources.utils";
import { Badge } from "@/modules/ui/components/Badge";

interface ExpeditionCardProps {
  type: string;
  data: ExpeditionResource;
}

export const ExpeditionCard: React.FC<ExpeditionCardProps> = ({
  type,
  data,
}) => {
  const tick = useAtomValue(currentTick);
  const name = getResourceDisplayName(type);

  const allDone = useMemo(
    () => isPastDateTime(data.earliestFinishAt, tick),
    [data.earliestFinishAt, tick],
  );

  const timeRemaining = useMemo(
    () => formatTimeRemaining(data.earliestFinishAt, tick),
    [data.earliestFinishAt, tick],
  );

  return (
    <div className="rounded-lg bg-zinc-50 p-3 dark:bg-zinc-800/50">
      <div className="mb-1 text-xs font-medium text-zinc-500 dark:text-zinc-400">
        {name}
      </div>
      <div className="text-xl font-bold text-zinc-950 dark:text-white">
        {data.currentExpeditions}{" "}
        <span className="text-sm font-normal text-zinc-500 dark:text-zinc-400">
          / {data.maxExpeditions} active
        </span>
      </div>
      <div className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
        {data.currentExpeditions > 0 ? (
          allDone ? (
            <Badge variant="success">Ready to collect!</Badge>
          ) : (
            `Next: ${timeRemaining}`
          )
        ) : (
          "No expeditions"
        )}
      </div>
    </div>
  );
};
