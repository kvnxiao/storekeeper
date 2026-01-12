import { useAtomValue } from "jotai";
import { useMemo } from "react";

import { currentTick } from "@/modules/core/core.tick";
import type { CooldownResource } from "@/modules/resources/resources.types";
import {
  formatTimeRemaining,
  getResourceDisplayName,
} from "@/modules/resources/resources.utils";
import { Badge } from "@/modules/ui/components/Badge";

interface CooldownCardProps {
  type: string;
  data: CooldownResource;
}

export const CooldownCard: React.FC<CooldownCardProps> = ({ type, data }) => {
  const tick = useAtomValue(currentTick);
  const name = getResourceDisplayName(type);

  const timeRemaining = useMemo(
    () => formatTimeRemaining(data.readyAt, tick),
    [data.readyAt, tick],
  );

  return (
    <div className="rounded-lg bg-zinc-50 p-3 dark:bg-zinc-800/50">
      <div className="mb-1 text-xs font-medium text-zinc-500 dark:text-zinc-400">
        {name}
      </div>
      <div className="text-xl font-bold">
        {data.isReady ? (
          <Badge variant="success">Ready!</Badge>
        ) : (
          <Badge variant="warning">{timeRemaining}</Badge>
        )}
      </div>
    </div>
  );
};
