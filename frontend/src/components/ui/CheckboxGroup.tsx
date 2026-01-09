import type React from "react";
import {
  CheckboxGroup as AriaCheckboxGroup,
  type CheckboxGroupProps as AriaCheckboxGroupProps,
  composeRenderProps,
} from "react-aria-components";
import { tv } from "tailwind-variants";

const checkboxGroupStyle = tv({
  base: "flex flex-col gap-2",
});

export interface CheckboxGroupProps extends AriaCheckboxGroupProps {
  className?: string;
}

export const CheckboxGroup: React.FC<CheckboxGroupProps> = ({
  className,
  ...props
}) => {
  return (
    <AriaCheckboxGroup
      {...props}
      className={composeRenderProps(className, (cn) =>
        checkboxGroupStyle({ className: cn }),
      )}
    />
  );
};
