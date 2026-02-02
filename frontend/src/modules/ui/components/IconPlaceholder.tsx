import type React from "react";
import { tv, type VariantProps } from "tailwind-variants";

const iconPlaceholderStyle = tv({
  base: "shrink-0 rounded bg-zinc-200 dark:bg-zinc-700",
  variants: {
    size: {
      sm: "size-5",
      md: "size-7",
    },
  },
  defaultVariants: {
    size: "md",
  },
});

type IconPlaceholderStyleProps = VariantProps<typeof iconPlaceholderStyle>;

export interface IconPlaceholderProps
  extends React.HTMLAttributes<HTMLDivElement>,
    IconPlaceholderStyleProps {}

export const IconPlaceholder: React.FC<IconPlaceholderProps> = ({
  size,
  className,
  ...props
}) => {
  return (
    <div className={iconPlaceholderStyle({ size, className })} {...props} />
  );
};
