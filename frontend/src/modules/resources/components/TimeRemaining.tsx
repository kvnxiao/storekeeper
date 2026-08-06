import { Show, type VoidComponent } from "solid-js";
import { Tooltip } from "@/modules/ui/components/Tooltip";

const interactiveStyle = [
  "rounded bg-zinc-100 px-1 text-zinc-700 transition-colors",
  "hover:bg-zinc-200 dark:bg-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-600",
].join(" ");

export interface TimeRemainingProps {
  relativeTime: string;
  absoluteTime: string | null;
  /** Skip background styling (use when nested inside Badge) */
  plain?: boolean;
}

export const TimeRemaining: VoidComponent<TimeRemainingProps> = (props) => {
  return (
    <Show when={props.absoluteTime} fallback={<time>{props.relativeTime}</time>}>
      <Tooltip
        content={props.absoluteTime}
        triggerClass={props.plain ? undefined : interactiveStyle}
      >
        {props.relativeTime}
      </Tooltip>
    </Show>
  );
};
