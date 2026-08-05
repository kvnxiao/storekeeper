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
  data?: StaminaResource | null;
  formattedTime: FormattedTime;
  isRefreshing?: boolean;
}

/**
 * The card shell, icon, and name are shared between the loading and loaded
 * states; only the value regions swap. Swapping the whole subtree when data
 * first arrives makes every card flash at once.
 */
export const StaminaCard: VoidComponent<StaminaCardProps> = (props) => {
  return (
    <div class="animate-card-in">
      <div
        class={cn(
          "rounded-lg bg-zinc-50 p-2 transition-transform hover:translate-x-0.5 dark:bg-zinc-700",
          (props.isRefreshing || !props.data) && "mask-shimmer",
        )}
      >
        <div class="flex items-center gap-2">
          <ResourceIcon src={props.iconPath} />
          <div class="flex min-w-0 flex-1 items-baseline justify-between gap-2">
            <span class="truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
              {props.name}
            </span>
            <Show
              when={props.data}
              // h-5 matches text-sm line-height (1.25rem = 20px)
              fallback={<div class="h-5 w-12 rounded bg-zinc-200 dark:bg-zinc-600" />}
            >
              {(data) => (
                <span class="shrink-0 text-sm tabular-nums text-zinc-950 dark:text-white">
                  <span class="font-semibold">{data().current}</span>
                  <span class="text-zinc-500 dark:text-zinc-400">/{data().max}</span>
                </span>
              )}
            </Show>
          </div>
        </div>
        <Show
          when={props.data}
          fallback={
            <>
              <div class="mt-1.5 h-1 w-full rounded bg-zinc-200 dark:bg-zinc-600" />
              {/* h-4 matches text-xs line-height (1rem = 16px) */}
              <div class="mt-1 h-4 w-24 rounded bg-zinc-200 dark:bg-zinc-600" />
            </>
          }
        >
          {(data) => (
            <>
              <ProgressBar
                value={Math.min((data().current / data().max) * 100, 100)}
                fillColor="linear-gradient(to right, #3b82f6, #f59e0b, #ef4444)"
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
            </>
          )}
        </Show>
      </div>
    </div>
  );
};
