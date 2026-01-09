import { Badge } from "@/components/ui/Badge";
import type { CooldownResource } from "@/types";
import { formatTime, getResourceDisplayName } from "@/utils/resourceFormatting";

interface CooldownCardProps {
  type: string;
  data: CooldownResource;
}

export const CooldownCard: React.FC<CooldownCardProps> = ({ type, data }) => {
  const name = getResourceDisplayName(type);

  return (
    <div className="rounded-lg bg-gray-50 p-3 dark:bg-gray-700">
      <div className="mb-1 text-xs font-medium text-gray-500 dark:text-gray-400">
        {name}
      </div>
      <div className="text-xl font-bold">
        {data.isReady ? (
          <Badge variant="success">Ready!</Badge>
        ) : (
          <Badge variant="warning">{formatTime(data.readyAt)}</Badge>
        )}
      </div>
    </div>
  );
};
