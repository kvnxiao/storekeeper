import { ProgressBar } from "@/components/ui/ProgressBar";
import type { StaminaResource } from "@/types";
import {
  formatTime,
  getProgressColor,
  getResourceDisplayName,
} from "@/utils/resourceFormatting";

interface StaminaCardProps {
  type: string;
  data: StaminaResource;
}

export const StaminaCard: React.FC<StaminaCardProps> = ({ type, data }) => {
  const name = getResourceDisplayName(type);
  const percentage = (data.current / data.max) * 100;
  const isFull = data.current >= data.max;
  const progressColor = getProgressColor(percentage, isFull);

  return (
    <div className="rounded-lg bg-gray-50 p-3 dark:bg-gray-700">
      <div className="mb-1 text-xs font-medium text-gray-500 dark:text-gray-400">
        {name}
      </div>
      <div className="text-xl font-bold text-gray-900 dark:text-white">
        {data.current}{" "}
        <span className="text-sm font-normal text-gray-500 dark:text-gray-400">
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
      <div className="mt-1 text-xs text-gray-500 dark:text-gray-400">
        {isFull ? "Full!" : `Full in ${formatTime(data.fullAt)}`}
      </div>
    </div>
  );
};
