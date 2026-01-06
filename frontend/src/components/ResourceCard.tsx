import { ProgressBar } from "react-aria-components";

import type {
  CooldownResource,
  ExpeditionResource,
  GameId,
  StaminaResource,
} from "@/types";
import {
  isCooldownResource,
  isExpeditionResource,
  isStaminaResource,
} from "@/types";

interface Props {
  gameId: GameId;
  type: string;
  data: unknown;
}

/** Formats a datetime string to human-readable time remaining */
function formatTime(datetime: string | null | undefined): string {
  if (!datetime) return "Full";

  const target = new Date(datetime);
  const now = new Date();
  const diffMs = target.getTime() - now.getTime();

  if (diffMs <= 0) return "Full";

  const seconds = Math.floor(diffMs / 1000);
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);

  if (hours > 0) return `${hours}h ${mins}m`;
  return `${mins}m`;
}

/** Checks if a datetime is in the past */
function isPastDateTime(datetime: string | null | undefined): boolean {
  if (!datetime) return true;
  const target = new Date(datetime);
  const now = new Date();
  return target.getTime() <= now.getTime();
}

/** Maps resource type to display name */
function getResourceDisplayName(type: string): string {
  const names: Record<string, string> = {
    resin: "Original Resin",
    parametric_transformer: "Transformer",
    realm_currency: "Realm Currency",
    expeditions: "Expeditions",
    trailblaze_power: "Trailblaze Power",
    battery: "Battery",
    waveplates: "Waveplates",
  };
  return names[type] || type;
}

/** Get progress bar color based on percentage */
function getProgressColor(percentage: number, isFull: boolean): string {
  if (isFull) return "bg-green-500";
  if (percentage >= 50) return "bg-blue-500";
  if (percentage >= 25) return "bg-yellow-500";
  return "bg-red-500";
}

const StaminaCard: React.FC<{ data: StaminaResource }> = ({ data }) => {
  const percentage = (data.current / data.max) * 100;
  const isFull = data.current >= data.max;
  const progressColor = getProgressColor(percentage, isFull);

  return (
    <>
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
        className="mt-2"
        aria-label="Resource progress"
      >
        {({ percentage: pct }) => (
          <div className="h-2 bg-gray-200 dark:bg-gray-600 rounded-full overflow-hidden">
            <div
              className={`h-full transition-all duration-300 ${progressColor}`}
              style={{ width: `${pct}%` }}
            />
          </div>
        )}
      </ProgressBar>
      <div className="mt-1 text-xs text-gray-500 dark:text-gray-400">
        {isFull ? "Full!" : `Full in ${formatTime(data.fullAt)}`}
      </div>
    </>
  );
};

const CooldownCard: React.FC<{ data: CooldownResource }> = ({ data }) => {
  return (
    <div className="text-xl font-bold">
      {data.isReady ? (
        <span className="text-green-600 dark:text-green-400">Ready!</span>
      ) : (
        <span className="text-yellow-600 dark:text-yellow-400">
          {formatTime(data.readyAt)}
        </span>
      )}
    </div>
  );
};

const ExpeditionCard: React.FC<{ data: ExpeditionResource }> = ({ data }) => {
  const allDone = isPastDateTime(data.earliestFinishAt);

  return (
    <>
      <div className="text-xl font-bold text-gray-900 dark:text-white">
        {data.currentExpeditions}{" "}
        <span className="text-sm font-normal text-gray-500 dark:text-gray-400">
          / {data.maxExpeditions} active
        </span>
      </div>
      <div className="mt-1 text-xs text-gray-500 dark:text-gray-400">
        {data.currentExpeditions > 0 ? (
          allDone ? (
            <span className="text-green-600 dark:text-green-400">
              Ready to collect!
            </span>
          ) : (
            `Next: ${formatTime(data.earliestFinishAt)}`
          )
        ) : (
          "No expeditions"
        )}
      </div>
    </>
  );
};

export const ResourceCard: React.FC<Props> = ({ type, data }) => {
  const name = getResourceDisplayName(type);

  return (
    <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
      <div className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">
        {name}
      </div>

      {isStaminaResource(data) && <StaminaCard data={data} />}
      {isCooldownResource(data) && <CooldownCard data={data} />}
      {isExpeditionResource(data) && <ExpeditionCard data={data} />}
    </div>
  );
};
