import { createLink, type LinkComponent } from "@tanstack/react-router";
import type React from "react";
import {
  composeRenderProps,
  Link as RACLink,
  type LinkProps as RACLinkProps,
} from "react-aria-components";

import {
  type ButtonStyleProps,
  buttonStyle,
} from "@/modules/ui/components/Button";

interface ButtonLinkProps extends RACLinkProps, ButtonStyleProps {
  className?: string;
}

/**
 * Base ButtonLink component with React Aria Link + button styling.
 */
const ButtonLinkBase: React.FC<ButtonLinkProps> = ({
  variant,
  color,
  size,
  className,
  children,
  ...props
}) => {
  // Only apply color for solid variant
  const effectiveColor = variant === "solid" || !variant ? color : undefined;

  return (
    <RACLink
      {...props}
      className={composeRenderProps(className, (cn) =>
        buttonStyle({ variant, color: effectiveColor, size, className: cn }),
      )}
    >
      {children}
    </RACLink>
  );
};

/**
 * Link component styled as a button, integrated with TanStack Router.
 * Use this for navigation that should look like a button.
 */
export const ButtonLink: LinkComponent<typeof ButtonLinkBase> =
  createLink(ButtonLinkBase);
