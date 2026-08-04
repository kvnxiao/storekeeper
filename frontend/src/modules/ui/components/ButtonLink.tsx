import { Link, type LinkComponentProps } from "@tanstack/solid-router";
import { type Component, splitProps } from "solid-js";
import { type ButtonStyleProps, buttonStyle } from "@/modules/ui/components/Button";

export interface ButtonLinkProps extends LinkComponentProps, ButtonStyleProps {}

/**
 * TanStack Router link styled as a button.
 * Use this for navigation that should look like a button.
 */
export const ButtonLink: Component<ButtonLinkProps> = (props) => {
  const [local, rest] = splitProps(props, ["variant", "color", "size", "class", "children"]);

  // Only apply color for solid variant
  const effectiveColor = () =>
    local.variant === "solid" || !local.variant ? local.color : undefined;

  return (
    <Link
      {...rest}
      class={buttonStyle({
        variant: local.variant,
        color: effectiveColor(),
        size: local.size,
        class: local.class,
      })}
    >
      {local.children}
    </Link>
  );
};
