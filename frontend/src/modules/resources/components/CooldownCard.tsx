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
  data?: CooldownResource;
  formattedTime: FormattedTime;
  isRefreshing?: boolean;
}

export const CooldownCard: VoidComponent<CooldownCardProps> = (props) => {
  return (
    <div class="animate-card-in">
      <Show
        when={props.data}
        fallback={
          // Loading state - show icon + name with shimmer badge placeholder
          <div class="mask-shimmer rounded-lg bg-zinc-50 p-2 transition-transform hover:translate-x-0.5 dark:bg-zinc-700">
            <div class="flex items-center gap-2">
              <ResourceIcon src={props.iconPath} size="md" />
              <span class="min-w-0 flex-1 truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
                {props.name}
              </span>
              <div class="h-5 w-14 rounded-full bg-zinc-200 dark:bg-zinc-600" />
            </div>
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
              <span class="min-w-0 flex-1 truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
                {props.name}
              </span>
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
            </div>
            <div class="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
              <Show when={!data().isReady} fallback={m.cooldown_ready()}>
                {m.cooldown_ready_in()}{" "}
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
