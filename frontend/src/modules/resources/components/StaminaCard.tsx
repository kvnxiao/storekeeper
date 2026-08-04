import { Show, type VoidComponent } from "solid-js";
import { ResourceIcon } from "@/modules/resources/components/ResourceIcon";
import { TimeRemaining } from "@/modules/resources/components/TimeRemaining";
import type { FormattedTime, StaminaResource } from "@/modules/resources/resources.types";
import { ProgressBar } from "@/modules/ui/components/ProgressBar";
import { cn } from "@/modules/ui/ui.styles";
import * as m from "@/paraglide/messages";

export interface StaminaCardProps {
  iconPath: string;
  name: string;
  data?: StaminaResource;
  formattedTime: FormattedTime;
  isRefreshing?: boolean;
}

export const StaminaCard: VoidComponent<StaminaCardProps> = (props) => {
  const percentage = () => {
    const data = props.data;
    return data ? Math.min((data.current / data.max) * 100, 100) : 0;
  };

  return (
    <div class="animate-card-in">
      <Show
        when={props.data}
        fallback={
          // Loading state - show icon + name with shimmer placeholders
          <div class="mask-shimmer rounded-lg bg-zinc-50 p-2 transition-transform hover:translate-x-0.5 dark:bg-zinc-700">
            <div class="flex items-center gap-2">
              <ResourceIcon src={props.iconPath} size="md" />
              <div class="flex min-w-0 flex-1 items-baseline justify-between gap-2">
                <span class="truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
                  {props.name}
                </span>
                {/* h-5 matches text-sm line-height (1.25rem = 20px) */}
                <div class="h-5 w-12 rounded bg-zinc-200 dark:bg-zinc-600" />
              </div>
            </div>
            <div class="mt-1.5 h-1 w-full rounded bg-zinc-200 dark:bg-zinc-600" />
            {/* h-4 matches text-xs line-height (1rem = 16px) */}
            <div class="mt-1 h-4 w-24 rounded bg-zinc-200 dark:bg-zinc-600" />
          </div>
        }
      >
        {(data) => (
          <div
            class={cn(
              "rounded-lg bg-zinc-50 p-2 transition-transform hover:translate-x-0.5 dark:bg-zinc-700",
              props.isRefreshing && "mask-shimmer",
            )}
          >
            <div class="flex items-center gap-2">
              <ResourceIcon src={props.iconPath} size="md" />
              <div class="flex min-w-0 flex-1 items-baseline justify-between gap-2">
                <span class="truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
                  {props.name}
                </span>
                <span class="shrink-0 text-sm tabular-nums text-zinc-950 dark:text-white">
                  <span class="font-semibold">{data().current}</span>
                  <span class="text-zinc-500 dark:text-zinc-400">/{data().max}</span>
                </span>
              </div>
            </div>
            <ProgressBar
              value={percentage()}
              minValue={0}
              maxValue={100}
              fillColor="linear-gradient(to right, #3b82f6, #f59e0b, #ef4444)"
              size="xs"
              class="mt-1.5"
              aria-label={m.stamina_progress_label({
                name: props.name,
                current: String(data().current),
                max: String(data().max),
              })}
            />
            <div class="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
              <Show when={data().current < data().max} fallback={m.stamina_full()}>
                {m.stamina_full_in()}{" "}
                <TimeRemaining
                  relativeTime={props.formattedTime.relativeTime}
                  absoluteTime={props.formattedTime.absoluteTime}
                />
              </Show>
            </div>
          </div>
        )}
      </Show>
    </div>
  );
};
