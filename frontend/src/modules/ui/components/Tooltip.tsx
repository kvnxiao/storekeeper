import * as TooltipPrimitive from "@kobalte/core/tooltip";
import { type JSX, type ParentComponent, splitProps } from "solid-js";
import { type ButtonProps, buttonClass } from "@/modules/ui/components/Button";

const tooltipStyle =
  "z-50 rounded bg-zinc-100 px-2 py-1 text-xs text-zinc-900 shadow-md ring-1 ring-zinc-300 dark:bg-zinc-800 dark:text-white dark:ring-zinc-600";

const DEFAULT_PLACEMENT = "top";
const DEFAULT_OPEN_DELAY_MS = 300;
// Kobalte adds half the arrow's height to this value, so 0 already clears the
// arrow and anything here is the gap between the trigger and the arrow tip.
const DEFAULT_GUTTER_PX = 4;

export interface TooltipBehaviorProps {
  /** Side of the trigger the bubble prefers. Defaults to `top`. */
  placement?: TooltipPrimitive.TooltipRootProps["placement"];
  /** Hover time in milliseconds before the bubble opens. Defaults to 300. */
  openDelay?: number;
  /** Gap in pixels between the trigger and the bubble. Defaults to 8. */
  gutter?: number;
}

export interface TooltipProps extends TooltipBehaviorProps {
  /** Tooltip bubble content; `children` is the trigger content. */
  content: JSX.Element;
  triggerClass?: string;
}

export const Tooltip: ParentComponent<TooltipProps> = (props) => {
  return (
    <TooltipPrimitive.Root
      placement={props.placement ?? DEFAULT_PLACEMENT}
      openDelay={props.openDelay ?? DEFAULT_OPEN_DELAY_MS}
      gutter={props.gutter ?? DEFAULT_GUTTER_PX}
    >
      {/* Kobalte renders the trigger as a bare <button>, which defaults to
          submit inside a form. A tooltip must never submit anything. */}
      <TooltipPrimitive.Trigger type="button" class={props.triggerClass}>
        {props.children}
      </TooltipPrimitive.Trigger>
      <TooltipPrimitive.Portal>
        <TooltipPrimitive.Content class={tooltipStyle}>
          <TooltipPrimitive.Arrow />
          {props.content}
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    </TooltipPrimitive.Root>
  );
};

export interface TooltipButtonProps
  // `type` is fixed: a tooltip trigger inside a form must never submit it.
  extends Omit<JSX.ButtonHTMLAttributes<HTMLButtonElement>, "type">, TooltipBehaviorProps {
  /** Tooltip bubble content. An icon-only trigger still needs its own `aria-label`. */
  tooltip: JSX.Element;
  variant?: ButtonProps["variant"];
  color?: ButtonProps["color"];
  size?: ButtonProps["size"];
}

// The trigger carries the button's own styles rather than wrapping a Button:
// the trigger is already a button, and Kobalte anchors the bubble to the
// element it renders itself.
export const TooltipButton: ParentComponent<TooltipButtonProps> = (props) => {
  const [style, local, rest] = splitProps(
    props,
    ["variant", "color", "size", "class"],
    ["tooltip", "placement", "openDelay", "gutter", "children"],
  );

  return (
    <TooltipPrimitive.Root
      placement={local.placement ?? DEFAULT_PLACEMENT}
      openDelay={local.openDelay ?? DEFAULT_OPEN_DELAY_MS}
      gutter={local.gutter ?? DEFAULT_GUTTER_PX}
    >
      <TooltipPrimitive.Trigger type="button" class={buttonClass(style)} {...rest}>
        {local.children}
      </TooltipPrimitive.Trigger>
      <TooltipPrimitive.Portal>
        <TooltipPrimitive.Content class={tooltipStyle}>
          <TooltipPrimitive.Arrow />
          {local.tooltip}
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    </TooltipPrimitive.Root>
  );
};
