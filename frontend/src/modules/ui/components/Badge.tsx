import type React from "react";
import { tv, type VariantProps } from "tailwind-variants";

// Catalyst-style badge with hover states
const badgeStyle = tv({
  base: "inline-flex items-center gap-x-1.5 rounded-md px-1.5 py-0.5 text-sm font-medium sm:text-xs",
  variants: {
    variant: {
      default:
        "bg-zinc-600/10 text-zinc-700 hover:bg-zinc-600/20 dark:bg-white/5 dark:text-zinc-400 dark:hover:bg-white/10",
      secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
      destructive:
        "bg-red-500/15 text-red-700 hover:bg-red-500/25 dark:bg-red-500/10 dark:text-red-400 dark:hover:bg-red-500/20",
      outline: "border border-border text-foreground hover:bg-accent",
      success:
        "bg-green-500/15 text-green-700 hover:bg-green-500/25 dark:bg-green-500/10 dark:text-green-400 dark:hover:bg-green-500/20",
      warning:
        "bg-amber-400/20 text-amber-700 hover:bg-amber-400/30 dark:bg-amber-400/10 dark:text-amber-400 dark:hover:bg-amber-400/15",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

type BadgeStyleProps = VariantProps<typeof badgeStyle>;

export interface BadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    BadgeStyleProps {}

export const Badge: React.FC<BadgeProps> = ({
  variant,
  className,
  ...props
}) => {
  return <span className={badgeStyle({ variant, className })} {...props} />;
};
