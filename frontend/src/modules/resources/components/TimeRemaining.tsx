import { Show, type VoidComponent } from "solid-js";
import { tv } from "tailwind-variants";
import { Tooltip } from "@/modules/ui/components/Tooltip";
import { cn } from "@/modules/ui/ui.styles";

const interactiveStyle = tv({
  base: [
    "rounded bg-zinc-100 px-1 text-zinc-700 transition-colors",
    "hover:bg-zinc-200 dark:bg-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-600",
  ],
});

export interface TimeRemainingProps {
  relativeTime: string;
  absoluteTime: string | null;
  class?: string;
  /** Skip background styling (use when nested inside Badge) */
  plain?: boolean;
}

export const TimeRemaining: VoidComponent<TimeRemainingProps> = (props) => {
  return (
    <Show
      when={props.absoluteTime}
      fallback={<time class={props.class}>{props.relativeTime}</time>}
    >
      <Tooltip
        content={props.absoluteTime}
        triggerClass={cn(!props.plain && interactiveStyle(), props.class)}
      >
        {props.relativeTime}
      </Tooltip>
    </Show>
  );
};
