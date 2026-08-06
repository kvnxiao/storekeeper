import { Show, type VoidComponent } from "solid-js";
import { ResourceCard } from "@/modules/resources/components/ResourceCard";
import { TimeRemaining } from "@/modules/resources/components/TimeRemaining";
import type { CooldownResource, FormattedTime } from "@/modules/resources/resources.types";
import { Badge } from "@/modules/ui/components/Badge";
import { Skeleton } from "@/modules/ui/components/Skeleton";
import * as m from "@/paraglide/messages";

export interface CooldownCardProps {
  iconPath: string;
  name: string;
  data?: CooldownResource | null;
  formattedTime: FormattedTime;
}

export const CooldownCard: VoidComponent<CooldownCardProps> = (props) => {
  return (
    <ResourceCard
      iconPath={props.iconPath}
      name={props.name}
      hasData={Boolean(props.data)}
      trailing={
        <Show when={props.data} fallback={<Skeleton class="h-5 w-14 rounded-full" />}>
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
      }
    >
      <Show when={props.data} fallback={<Skeleton class="mt-1 h-4 w-24" />}>
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
    </ResourceCard>
  );
};
