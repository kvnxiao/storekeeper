import * as TooltipPrimitive from "@kobalte/core/tooltip";
import { type JSX, type ParentComponent, splitProps } from "solid-js";
import { type ButtonProps, buttonClass } from "@/modules/ui/components/Button";

const tooltipStyle =
  "z-50 rounded bg-zinc-100 px-2 py-1 text-xs text-zinc-900 shadow-md ring-1 ring-zinc-300 dark:bg-zinc-800 dark:text-white dark:ring-zinc-600";

const DEFAULT_PLACEMENT = "top";
const DEFAULT_OPEN_DELAY_MS = 300;
// Kobalte adds gutter to half the arrow height; zero places the arrow tip
// against the trigger.
const DEFAULT_GUTTER_PX = 0;

// Use a 12px arrow instead of Kobalte's 30px default.
const ARROW_SIZE_PX = 12;

export interface TooltipBehaviorProps {
  /** Side of the trigger the bubble prefers. Defaults to `top`. */
  placement?: TooltipPrimitive.TooltipRootProps["placement"];
  /** Hover time in milliseconds before the bubble opens. Defaults to 300. */
  openDelay?: number;
  /** Gap in pixels between the trigger and the bubble. Defaults to 0. */
  gutter?: number;
}

export interface TooltipProps extends TooltipBehaviorProps {
  /** Tooltip content; `children` is the trigger content. */
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
          <TooltipPrimitive.Arrow size={ARROW_SIZE_PX} />
          {props.content}
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    </TooltipPrimitive.Root>
  );
};

export interface TooltipButtonProps
  // Fix `type` to keep tooltip triggers from submitting an enclosing form.
  extends Omit<JSX.ButtonHTMLAttributes<HTMLButtonElement>, "type">, TooltipBehaviorProps {
  /** Tooltip content. Icon-only triggers require an `aria-label`. */
  tooltip: JSX.Element;
  variant?: ButtonProps["variant"];
  color?: ButtonProps["color"];
  size?: ButtonProps["size"];
}

// Apply button styles to the trigger because Kobalte anchors the bubble to that
// element.
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
          <TooltipPrimitive.Arrow size={ARROW_SIZE_PX} />
          {local.tooltip}
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    </TooltipPrimitive.Root>
  );
};
