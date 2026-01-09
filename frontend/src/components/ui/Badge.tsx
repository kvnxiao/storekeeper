import type React from "react";
import { tv, type VariantProps } from "tailwind-variants";

const badgeStyle = tv({
  base: "inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium transition-colors",
  variants: {
    variant: {
      default: "bg-primary/10 text-primary",
      secondary: "bg-secondary text-secondary-foreground",
      destructive: "bg-destructive/10 text-destructive",
      outline: "border border-border text-foreground",
      success: "bg-green-500/10 text-green-600 dark:text-green-400",
      warning: "bg-amber-500/10 text-amber-600 dark:text-amber-400",
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
