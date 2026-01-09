import type React from "react";
import {
  Button as AriaButton,
  type ButtonProps as AriaButtonProps,
  composeRenderProps,
} from "react-aria-components";
import { tv, type VariantProps } from "tailwind-variants";

const buttonStyle = tv({
  base: "inline-flex items-center justify-center gap-2 rounded-lg font-medium transition-colors cursor-default disabled:pointer-events-none disabled:opacity-50 outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
  variants: {
    variant: {
      solid:
        "bg-primary text-primary-foreground shadow-sm hovered:bg-primary/90 pressed:bg-primary/80",
      outline:
        "border border-input bg-background hovered:bg-accent hovered:text-accent-foreground pressed:bg-accent/80",
      ghost:
        "hovered:bg-accent hovered:text-accent-foreground pressed:bg-accent/80",
      destructive:
        "bg-destructive text-destructive-foreground shadow-sm hovered:bg-destructive/90 pressed:bg-destructive/80",
      link: "text-primary underline-offset-4 hovered:underline",
    },
    size: {
      sm: "h-8 px-3 text-xs",
      md: "h-9 px-4 text-sm",
      lg: "h-10 px-6 text-base",
      icon: "size-9",
    },
  },
  defaultVariants: {
    variant: "solid",
    size: "md",
  },
});

type ButtonStyleProps = VariantProps<typeof buttonStyle>;

export interface ButtonProps extends AriaButtonProps, ButtonStyleProps {
  className?: string;
}

export const Button: React.FC<ButtonProps> = ({
  variant,
  size,
  className,
  ...props
}) => {
  return (
    <AriaButton
      {...props}
      className={composeRenderProps(className, (cn) =>
        buttonStyle({ variant, size, className: cn }),
      )}
    />
  );
};
