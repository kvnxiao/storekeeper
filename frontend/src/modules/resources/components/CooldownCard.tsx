import { Show, type VoidComponent } from "solid-js";
import { ResourceIcon } from "@/modules/resources/components/ResourceIcon";
import { TimeRemaining } from "@/modules/resources/components/TimeRemaining";
import type { CooldownResource, FormattedTime } from "@/modules/resources/resources.types";
import { Badge } from "@/modules/ui/components/Badge";
import { cn } from "@/modules/ui/ui.styles";
import * as m from "@/paraglide/messages";

export interface CooldownCardProps {
  iconPath: string;
  name: string;
  data?: CooldownResource | null;
  formattedTime: FormattedTime;
  isRefreshing?: boolean;
}

/**
 * The card shell, icon, and name are shared between the loading and loaded
 * states; only the value regions swap. Swapping the whole subtree when data
 * first arrives makes every card flash at once.
 */
export const CooldownCard: VoidComponent<CooldownCardProps> = (props) => {
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
          <span class="min-w-0 flex-1 truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
            {props.name}
          </span>
          <Show
            when={props.data}
            fallback={<div class="h-5 w-14 rounded-full bg-zinc-200 dark:bg-zinc-600" />}
          >
            {(data) => (
              <Show
                when={!data().isReady}
                fallback={<Badge variant="success">{m.cooldown_ready()}</Badge>}
              >
                <Badge variant="warning">
                  <TimeRemaining
                    relativeTime={props.formattedTime.relativeTime}
                    absoluteTime={props.formattedTime.absoluteTime}
                    plain
                  />
                </Badge>
              </Show>
            )}
          </Show>
        </div>
        <Show
          when={props.data}
          // h-4 matches text-xs line-height (1rem = 16px)
          fallback={<div class="mt-1 h-4 w-24 rounded bg-zinc-200 dark:bg-zinc-600" />}
        >
          {(data) => (
            <div class="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
              <Show when={!data().isReady} fallback={m.cooldown_ready()}>
                {m.cooldown_ready_in()}{" "}
                <TimeRemaining
                  relativeTime={props.formattedTime.relativeTime}
                  absoluteTime={props.formattedTime.absoluteTime}
                />
              </Show>
            </div>
          )}
        </Show>
      </div>
    </div>
  );
};
