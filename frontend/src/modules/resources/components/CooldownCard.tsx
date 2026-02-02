import { ResourceIcon } from "@/modules/resources/components/ResourceIcon";
import { TimeRemaining } from "@/modules/resources/components/TimeRemaining";
import { useFormattedTime } from "@/modules/resources/resources.hooks";
import type { CooldownResource } from "@/modules/resources/resources.types";
import { getResourceDisplayName } from "@/modules/resources/resources.utils";
import { Badge } from "@/modules/ui/components/Badge";

interface CooldownCardProps {
  type: string;
  data: CooldownResource;
}

export const CooldownCard: React.FC<CooldownCardProps> = ({ type, data }) => {
  const name = getResourceDisplayName(type);
  const { relativeTime, absoluteTime } = useFormattedTime(data.readyAt);

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
          <Badge variant="warning">
            <TimeRemaining
              relativeTime={relativeTime}
              absoluteTime={absoluteTime}
              plain
            />
          </Badge>
        )}
      </div>
      {!data.isReady && (
        <div className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
          Ready in{" "}
          <TimeRemaining
            relativeTime={relativeTime}
            absoluteTime={absoluteTime}
          />
        </div>
      )}
    </div>
  );
};
