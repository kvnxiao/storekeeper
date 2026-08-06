import { Link, type LinkComponentProps } from "@tanstack/solid-router";
import { type Component, splitProps } from "solid-js";
import { type ButtonStyleProps, buttonClass } from "@/modules/ui/components/Button";

export interface ButtonLinkProps extends LinkComponentProps, ButtonStyleProps {}

/**
 * TanStack Router link styled as a button.
 * Use this for navigation that should look like a button.
 */
export const ButtonLink: Component<ButtonLinkProps> = (props) => {
  const [local, rest] = splitProps(props, ["variant", "color", "size", "class", "children"]);

  return (
    <Link {...rest} class={buttonClass(local)}>
      {local.children}
    </Link>
  );
};
