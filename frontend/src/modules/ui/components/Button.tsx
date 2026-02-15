import { ArrowPathIcon } from "@heroicons/react/20/solid";
import type React from "react";
import {
  Button as AriaButton,
  type ButtonProps as AriaButtonProps,
  composeRenderProps,
} from "react-aria-components";
import { tv, type VariantProps } from "tailwind-variants";

// Catalyst-style button with layered pseudo-elements
// Exported for reuse in ButtonLink
export const buttonStyle = tv({
  base: [
    // Base layout
    "relative isolate inline-flex items-center justify-center gap-x-2 rounded-lg border text-sm font-semibold",
    // Sizing
    "px-3 py-1.5",
    // Focus (using React Aria states)
    "outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500",
    // Disabled
    "disabled:opacity-50 disabled:cursor-not-allowed",
    // Cursor
    "cursor-pointer",
    // Touch target (44x44px hit area on touch devices)
    "touch-target",
  ],
  variants: {
    variant: {
      solid: [
        // Optical border, implemented as the button background to avoid corner artifacts
        "border-transparent bg-(--btn-border)",
        // Dark mode: border rendered on `after`, bg is button bg
        "dark:bg-(--btn-bg)",
        // Button background on `before`, stacked on top of pseudo-border layer
        "before:absolute before:inset-0 before:-z-10 before:rounded-[calc(var(--radius-lg)-1px)] before:bg-(--btn-bg)",
        // Drop shadow on `before` layer
        "before:shadow-sm",
        // Hide `before` in dark mode
        "dark:before:hidden",
        // Dark mode: subtle white outline
        "dark:border-white/5",
        // Shim/overlay on `after` for hover state + highlight shadow
        "after:absolute after:inset-0 after:-z-10 after:rounded-[calc(var(--radius-lg)-1px)]",
        // Inner highlight shadow (top edge light bevel)
        "after:shadow-[inset_0_1px_--theme(--color-white/15%)]",
        // Hover overlay
        "hover:after:bg-(--btn-hover-overlay) pressed:after:bg-(--btn-hover-overlay)",
        // Dark mode: `after` expands to cover entire button
        "dark:after:-inset-px dark:after:rounded-lg",
        // Disabled
        "disabled:before:shadow-none disabled:after:shadow-none",
      ],
      outline: [
        "border-zinc-950/10 text-zinc-950 hover:bg-zinc-950/2.5 pressed:bg-zinc-950/5",
        "dark:border-white/15 dark:text-white dark:hover:bg-white/5 dark:pressed:bg-white/10",
      ],
      plain: [
        "border-transparent text-zinc-950 hover:bg-zinc-950/10 pressed:bg-zinc-950/15",
        "dark:text-white dark:hover:bg-white/15 dark:pressed:bg-white/20",
      ],
    },
    color: {
      "dark/zinc": [
        "text-white [--btn-bg:theme(colors.zinc.900)] [--btn-border:theme(colors.zinc.950/90%)] [--btn-hover-overlay:theme(colors.white/10%)]",
        "dark:text-white dark:[--btn-bg:theme(colors.zinc.600)] dark:[--btn-hover-overlay:theme(colors.white/5%)]",
      ],
      light: [
        "text-zinc-950 [--btn-bg:white] [--btn-border:theme(colors.zinc.950/10%)] [--btn-hover-overlay:theme(colors.zinc.950/2.5%)]",
        "dark:text-white dark:[--btn-bg:theme(colors.zinc.800)] dark:[--btn-hover-overlay:theme(colors.white/5%)]",
      ],
      zinc: [
        "text-white [--btn-bg:theme(colors.zinc.600)] [--btn-border:theme(colors.zinc.700/90%)] [--btn-hover-overlay:theme(colors.white/10%)]",
        "dark:[--btn-hover-overlay:theme(colors.white/5%)]",
      ],
      blue: [
        "text-white [--btn-bg:theme(colors.blue.600)] [--btn-border:theme(colors.blue.700/90%)] [--btn-hover-overlay:theme(colors.white/10%)]",
      ],
      red: [
        "text-white [--btn-bg:theme(colors.red.600)] [--btn-border:theme(colors.red.700/90%)] [--btn-hover-overlay:theme(colors.white/10%)]",
      ],
      green: [
        "text-white [--btn-bg:theme(colors.green.600)] [--btn-border:theme(colors.green.700/90%)] [--btn-hover-overlay:theme(colors.white/10%)]",
      ],
    },
    size: {
      sm: "px-2.5 py-1 text-xs",
      md: "px-3 py-1.5 text-sm",
      lg: "px-4 py-2 text-base",
    },
  },
  compoundVariants: [
    // Only apply color when variant is solid
    {
      variant: "solid",
      color: "dark/zinc",
      class: "",
    },
  ],
  defaultVariants: {
    variant: "solid",
    color: "dark/zinc",
    size: "md",
  },
});

export type ButtonStyleProps = VariantProps<typeof buttonStyle>;

export interface ButtonProps extends AriaButtonProps, ButtonStyleProps {
  className?: string;
  isPending?: boolean;
}

export const Button: React.FC<ButtonProps> = ({
  variant,
  color,
  size,
  className,
  children,
  isPending,
  isDisabled,
  ...props
}) => {
  // Only apply color for solid variant
  const effectiveColor = variant === "solid" || !variant ? color : undefined;

  return (
    <AriaButton
      {...props}
      isDisabled={isDisabled || isPending}
      className={composeRenderProps(className, (cn) =>
        buttonStyle({ variant, color: effectiveColor, size, className: cn }),
      )}
    >
      {composeRenderProps(children, (children) => (
        <>
          {isPending && <ArrowPathIcon className="h-4 w-4 animate-spin" />}
          {children}
        </>
      ))}
    </AriaButton>
  );
};
