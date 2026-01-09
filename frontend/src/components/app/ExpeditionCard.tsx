import { Badge } from "@/components/ui/Badge";
import type { ExpeditionResource } from "@/types";
import {
  formatTime,
  getResourceDisplayName,
  isPastDateTime,
} from "@/utils/resourceFormatting";

interface ExpeditionCardProps {
  type: string;
  data: ExpeditionResource;
}

export const ExpeditionCard: React.FC<ExpeditionCardProps> = ({
  type,
  data,
}) => {
  const name = getResourceDisplayName(type);
  const allDone = isPastDateTime(data.earliestFinishAt);

  return (
    <div className="rounded-lg bg-gray-50 p-3 dark:bg-gray-700">
      <div className="mb-1 text-xs font-medium text-gray-500 dark:text-gray-400">
        {name}
      </div>
      <div className="text-xl font-bold text-gray-900 dark:text-white">
        {data.currentExpeditions}{" "}
        <span className="text-sm font-normal text-gray-500 dark:text-gray-400">
          / {data.maxExpeditions} active
        </span>
      </div>
      <div className="mt-1 text-xs text-gray-500 dark:text-gray-400">
        {data.currentExpeditions > 0 ? (
          allDone ? (
            <Badge variant="success">Ready to collect!</Badge>
          ) : (
            `Next: ${formatTime(data.earliestFinishAt)}`
          )
        ) : (
          "No expeditions"
        )}
      </div>
    </div>
  );
};
