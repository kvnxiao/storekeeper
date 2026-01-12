import type React from "react";
import {
  Input as AriaInput,
  type InputProps as AriaInputProps,
  composeRenderProps,
} from "react-aria-components";
import { tv, type VariantProps } from "tailwind-variants";

const inputStyle = tv({
  base: "flex h-9 w-full rounded-lg border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50 outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background invalid:border-destructive invalid:ring-destructive",
  variants: {
    inputSize: {
      sm: "h-8 text-xs",
      md: "h-9 text-sm",
      lg: "h-10 text-base",
    },
  },
  defaultVariants: {
    inputSize: "md",
  },
});

type InputStyleProps = VariantProps<typeof inputStyle>;

export interface InputProps extends AriaInputProps, InputStyleProps {
  className?: string;
}

export const Input: React.FC<InputProps> = ({
  inputSize,
  className,
  ...props
}) => {
  return (
    <AriaInput
      {...props}
      className={composeRenderProps(className, (cn) =>
        inputStyle({ inputSize, className: cn }),
      )}
    />
  );
};
