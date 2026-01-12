import type React from "react";
import {
  TextArea as AriaTextArea,
  type TextAreaProps as AriaTextAreaProps,
  composeRenderProps,
} from "react-aria-components";
import { tv, type VariantProps } from "tailwind-variants";

const textareaStyle = tv({
  base: "flex min-h-[60px] w-full rounded-lg border border-input bg-background px-3 py-2 text-sm shadow-sm transition-colors placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50 outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background invalid:border-destructive invalid:ring-destructive resize-none",
  variants: {
    textareaSize: {
      sm: "min-h-[40px] text-xs",
      md: "min-h-[60px] text-sm",
      lg: "min-h-[80px] text-base",
    },
  },
  defaultVariants: {
    textareaSize: "md",
  },
});

type TextareaStyleProps = VariantProps<typeof textareaStyle>;

export interface TextareaProps extends AriaTextAreaProps, TextareaStyleProps {
  className?: string;
}

export const Textarea: React.FC<TextareaProps> = ({
  textareaSize,
  className,
  ...props
}) => {
  return (
    <AriaTextArea
      {...props}
      className={composeRenderProps(className, (cn) =>
        textareaStyle({ textareaSize, className: cn }),
      )}
    />
  );
};
