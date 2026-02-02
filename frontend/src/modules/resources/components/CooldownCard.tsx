import { useAtomValue } from "jotai";
import { useMemo } from "react";

import { atoms } from "@/modules/atoms";
import { ResourceIcon } from "@/modules/resources/components/ResourceIcon";
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
  const tick = useAtomValue(atoms.core.tick);
  const name = getResourceDisplayName(type);

  const timeRemaining = useMemo(
    () => formatTimeRemaining(data.readyAt, tick),
    [data.readyAt, tick],
  );

  return (
    <div className="rounded-lg bg-zinc-50 p-2 dark:bg-zinc-700">
      <div className="flex items-center gap-2">
        <ResourceIcon type={type} size="md" />
        <span className="min-w-0 flex-1 truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
          {name}
        </span>
        {data.isReady ? (
          <Badge variant="success">Ready!</Badge>
        ) : (
          <Badge variant="warning">{timeRemaining}</Badge>
        )}
      </div>
      {!data.isReady && (
        <div className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
          Ready in {timeRemaining}
        </div>
      )}
    </div>
  );
};
