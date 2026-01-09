import { CheckIcon, MinusIcon } from "@heroicons/react/16/solid";
import type React from "react";
import {
  Checkbox as AriaCheckbox,
  type CheckboxProps as AriaCheckboxProps,
  composeRenderProps,
} from "react-aria-components";
import { tv } from "tailwind-variants";

const checkboxStyle = tv({
  base: "group flex items-center gap-2 text-sm",
});

const checkboxBoxStyle = tv({
  base: "flex size-4 shrink-0 items-center justify-center rounded border border-primary shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 selected:bg-primary selected:text-primary-foreground",
});

export interface CheckboxProps extends AriaCheckboxProps {
  className?: string;
}

export const Checkbox: React.FC<CheckboxProps> = ({
  className,
  children,
  ...props
}) => {
  return (
    <AriaCheckbox
      {...props}
      className={composeRenderProps(className, (cn) =>
        checkboxStyle({ className: cn }),
      )}
    >
      {composeRenderProps(
        children,
        (children, { isSelected, isIndeterminate }) => (
          <>
            <div className={checkboxBoxStyle()}>
              {isIndeterminate ? (
                <MinusIcon className="size-3 text-current" aria-hidden="true" />
              ) : isSelected ? (
                <CheckIcon className="size-3 text-current" aria-hidden="true" />
              ) : null}
            </div>
            {children}
          </>
        ),
      )}
    </AriaCheckbox>
  );
};
