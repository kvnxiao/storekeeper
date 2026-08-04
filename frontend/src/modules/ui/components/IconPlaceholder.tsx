import { type JSX, splitProps, type VoidComponent } from "solid-js";
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
  extends JSX.HTMLAttributes<HTMLDivElement>, IconPlaceholderStyleProps {}

export const IconPlaceholder: VoidComponent<IconPlaceholderProps> = (props) => {
  const [local, rest] = splitProps(props, ["size", "class"]);
  return <div class={iconPlaceholderStyle({ size: local.size, class: local.class })} {...rest} />;
};
