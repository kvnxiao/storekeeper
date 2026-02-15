import type React from "react";
import {
  Switch as AriaSwitch,
  type SwitchProps as AriaSwitchProps,
  composeRenderProps,
} from "react-aria-components";
import { tv } from "tailwind-variants";

import { cn } from "@/modules/ui/ui.styles";

const trackStyle = tv({
  base: [
    "flex h-5 w-8 shrink-0 cursor-default items-center rounded-full px-px",
    "border border-transparent shadow-inner transition duration-200 ease-in-out",
    // Default (unselected)
    "bg-zinc-200 dark:bg-zinc-600",
    "group-pressed:bg-zinc-300 dark:group-pressed:bg-zinc-500",
    // Selected
    "group-selected:bg-zinc-700 group-selected:group-pressed:bg-zinc-800",
    "dark:group-selected:bg-zinc-300 dark:group-selected:group-pressed:bg-zinc-200",
    // Disabled
    "group-disabled:cursor-not-allowed group-disabled:bg-zinc-100 dark:group-disabled:bg-zinc-800",
    // Focus ring
    "outline-none group-focus-visible:ring-2 group-focus-visible:ring-ring group-focus-visible:ring-offset-2 group-focus-visible:ring-offset-background",
  ],
});

const handleStyle = tv({
  base: [
    "h-4 w-4 rounded-full bg-white shadow-xs transition duration-200 ease-in-out",
    "outline outline-1 -outline-offset-1 outline-transparent",
    "dark:bg-zinc-900",
    "translate-x-0 group-selected:translate-x-3",
    "group-disabled:bg-zinc-50 dark:group-disabled:bg-zinc-700",
  ],
});

export interface SwitchProps extends Omit<AriaSwitchProps, "children"> {
  children?: React.ReactNode;
  className?: string;
}

export const Switch: React.FC<SwitchProps> = ({
  children,
  className,
  ...props
}) => {
  return (
    <AriaSwitch
      {...props}
      className={composeRenderProps(
        className,
        (userClassName) =>
          cn(
            "group relative flex items-center gap-2 text-sm font-medium transition",
            "text-zinc-950 dark:text-white",
            "disabled:text-zinc-400 dark:disabled:text-zinc-500",
            userClassName,
          ) ?? "",
      )}
    >
      <div className={trackStyle()}>
        <span className={handleStyle()} />
      </div>
      {children}
    </AriaSwitch>
  );
};
