import * as TooltipPrimitive from "@kobalte/core/tooltip";
import type { JSX, ParentComponent } from "solid-js";
import { tv } from "tailwind-variants";

const tooltipStyle = tv({
  base: "z-50 rounded bg-zinc-100 px-2 py-1 text-xs text-zinc-900 shadow-md ring-1 ring-zinc-300 dark:bg-zinc-800 dark:text-white dark:ring-zinc-600",
});

export interface TooltipProps {
  /** Tooltip bubble content; `children` is the trigger content. */
  content: JSX.Element;
  placement?: "top" | "bottom" | "left" | "right";
  openDelay?: number;
  triggerClass?: string;
}

export const Tooltip: ParentComponent<TooltipProps> = (props) => {
  return (
    <TooltipPrimitive.Root
      placement={props.placement ?? "top"}
      openDelay={props.openDelay ?? 300}
      gutter={8}
    >
      <TooltipPrimitive.Trigger class={props.triggerClass}>
        {props.children}
      </TooltipPrimitive.Trigger>
      <TooltipPrimitive.Portal>
        <TooltipPrimitive.Content class={tooltipStyle()}>
          <TooltipPrimitive.Arrow />
          {props.content}
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    </TooltipPrimitive.Root>
  );
};
