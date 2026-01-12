import type React from "react";
import {
  Switch as AriaSwitch,
  type SwitchProps as AriaSwitchProps,
  composeRenderProps,
} from "react-aria-components";
import { tv, type VariantProps } from "tailwind-variants";

import { cn, focusRingStyle } from "@/modules/ui/ui.styles";

const trackStyle = tv({
  extend: focusRingStyle,
  base: [
    "flex h-5 w-8 shrink-0 cursor-default items-center rounded-full px-px",
    "border border-transparent shadow-inner transition duration-200 ease-in-out",
  ],
  variants: {
    isSelected: {
      false:
        "bg-zinc-200 group-pressed:bg-zinc-300 dark:bg-zinc-600 dark:group-pressed:bg-zinc-500",
      true: "bg-zinc-700 group-pressed:bg-zinc-800 dark:bg-zinc-300 dark:group-pressed:bg-zinc-200",
    },
    isDisabled: {
      true: "cursor-not-allowed bg-zinc-100 dark:bg-zinc-800",
    },
  },
});

const handleStyle = tv({
  base: [
    "h-4 w-4 rounded-full bg-white shadow-xs transition duration-200 ease-in-out",
    "outline outline-1 -outline-offset-1 outline-transparent",
    "dark:bg-zinc-900",
  ],
  variants: {
    isSelected: {
      false: "translate-x-0",
      true: "translate-x-3",
    },
    isDisabled: {
      true: "bg-zinc-50 dark:bg-zinc-700",
    },
  },
});

type SwitchStyleProps = VariantProps<typeof trackStyle>;

export interface SwitchProps
  extends Omit<AriaSwitchProps, "children">,
    SwitchStyleProps {
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
      className={composeRenderProps(className, (userClassName) =>
        cn(
          "group relative flex items-center gap-2 text-sm font-medium transition",
          "text-zinc-950 dark:text-white",
          "disabled:text-zinc-400 dark:disabled:text-zinc-500",
          userClassName,
        ),
      )}
    >
      {(renderProps) => (
        <>
          <div className={trackStyle(renderProps)}>
            <span className={handleStyle(renderProps)} />
          </div>
          {children}
        </>
      )}
    </AriaSwitch>
  );
};
