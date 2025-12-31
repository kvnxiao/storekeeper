import { Show } from "solid-js";

interface StaminaResource {
  current: number;
  max: number;
  seconds_until_full: number | null;
  regen_rate_seconds: number;
}

interface CooldownResource {
  is_ready: boolean;
  seconds_until_ready: number | null;
}

interface ExpeditionResource {
  current_expeditions: number;
  max_expeditions: number;
  earliest_finish_seconds: number | null;
}

interface Props {
  gameId: string;
  type: string;
  data: unknown;
}

function formatTime(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined || seconds <= 0) {
    return "Full";
  }

  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);

  if (hours > 0) {
    return `${hours}h ${mins}m`;
  }
  return `${mins}m`;
}

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

function isStaminaResource(data: unknown): data is StaminaResource {
  return (
    typeof data === "object" &&
    data !== null &&
    "current" in data &&
    "max" in data
  );
}

function isCooldownResource(data: unknown): data is CooldownResource {
  return (
    typeof data === "object" && data !== null && "is_ready" in data
  );
}

function isExpeditionResource(data: unknown): data is ExpeditionResource {
  return (
    typeof data === "object" &&
    data !== null &&
    "current_expeditions" in data
  );
}

function ResourceCard(props: Props) {
  const data = () => props.data;
  const name = () => getResourceDisplayName(props.type);

  return (
    <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
      <div class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">
        {name()}
      </div>

      <Show when={isStaminaResource(data())}>
        {(() => {
          const stamina = data() as StaminaResource;
          const percentage = (stamina.current / stamina.max) * 100;
          const isFull = stamina.current >= stamina.max;

          return (
            <>
              <div class="text-xl font-bold text-gray-900 dark:text-white">
                {stamina.current}{" "}
                <span class="text-sm font-normal text-gray-500 dark:text-gray-400">
                  / {stamina.max}
                </span>
              </div>
              <div class="mt-2 h-2 bg-gray-200 dark:bg-gray-600 rounded-full overflow-hidden">
                <div
                  class="h-full transition-all duration-300"
                  classList={{
                    "bg-green-500": isFull,
                    "bg-blue-500": !isFull && percentage >= 50,
                    "bg-yellow-500": !isFull && percentage < 50 && percentage >= 25,
                    "bg-red-500": !isFull && percentage < 25,
                  }}
                  style={{ width: `${Math.min(percentage, 100)}%` }}
                />
              </div>
              <div class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                {isFull ? "Full!" : `Full in ${formatTime(stamina.seconds_until_full)}`}
              </div>
            </>
          );
        })()}
      </Show>

      <Show when={isCooldownResource(data())}>
        {(() => {
          const cooldown = data() as CooldownResource;

          return (
            <div class="text-xl font-bold">
              <Show
                when={cooldown.is_ready}
                fallback={
                  <span class="text-yellow-600 dark:text-yellow-400">
                    {formatTime(cooldown.seconds_until_ready)}
                  </span>
                }
              >
                <span class="text-green-600 dark:text-green-400">Ready!</span>
              </Show>
            </div>
          );
        })()}
      </Show>

      <Show when={isExpeditionResource(data())}>
        {(() => {
          const expedition = data() as ExpeditionResource;
          const allDone = expedition.earliest_finish_seconds === 0;

          return (
            <>
              <div class="text-xl font-bold text-gray-900 dark:text-white">
                {expedition.current_expeditions}{" "}
                <span class="text-sm font-normal text-gray-500 dark:text-gray-400">
                  / {expedition.max_expeditions} active
                </span>
              </div>
              <div class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                <Show
                  when={expedition.current_expeditions > 0}
                  fallback="No expeditions"
                >
                  <Show
                    when={!allDone}
                    fallback={
                      <span class="text-green-600 dark:text-green-400">
                        Ready to collect!
                      </span>
                    }
                  >
                    Next: {formatTime(expedition.earliest_finish_seconds)}
                  </Show>
                </Show>
              </div>
            </>
          );
        })()}
      </Show>
    </div>
  );
}

export default ResourceCard;
