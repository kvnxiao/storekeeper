import { useAtomValue } from "jotai";
import { useMemo } from "react";

import { atoms } from "@/modules/atoms";
import { ResourceIcon } from "@/modules/resources/components/ResourceIcon";
import type { ExpeditionResource } from "@/modules/resources/resources.types";
import {
  formatTimeRemaining,
  getResourceDisplayName,
} from "@/modules/resources/resources.utils";
import { Badge } from "@/modules/ui/components/Badge";

interface ExpeditionCardProps {
  type: string;
  data: ExpeditionResource;
  allDone: boolean;
}

export const ExpeditionCard: React.FC<ExpeditionCardProps> = ({
  type,
  data,
  allDone,
}) => {
  const tick = useAtomValue(atoms.core.tick);
  const name = getResourceDisplayName(type);

  const timeRemaining = useMemo(
    () => formatTimeRemaining(data.earliestFinishAt, tick),
    [data.earliestFinishAt, tick],
  );

  return (
    <div className="rounded-lg bg-zinc-50 p-2 dark:bg-zinc-700">
      <div className="flex items-center gap-2">
        <ResourceIcon type={type} size="md" />
        <span className="min-w-0 flex-1 truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
          {name}
        </span>
        <span className="shrink-0 text-sm tabular-nums text-zinc-950 dark:text-white">
          <span className="font-semibold">{data.currentExpeditions}</span>
          <span className="text-zinc-500 dark:text-zinc-400">
            /{data.maxExpeditions}
          </span>
        </span>
      </div>
      <div className="mt-1 flex items-center justify-end text-xs text-zinc-500 dark:text-zinc-400">
        {data.currentExpeditions > 0 ? (
          allDone ? (
            <Badge variant="success">Ready!</Badge>
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
