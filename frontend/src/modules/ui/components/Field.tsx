import type React from "react";
import {
  FieldError as AriaFieldError,
  type FieldErrorProps as AriaFieldErrorProps,
  Label as AriaLabel,
  type LabelProps as AriaLabelProps,
  Text as AriaText,
  type TextProps as AriaTextProps,
  composeRenderProps,
  Group,
  type GroupProps,
} from "react-aria-components";
import { tv } from "tailwind-variants";

// Label
const labelStyle = tv({
  base: "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
});

export interface LabelProps extends AriaLabelProps {
  className?: string;
}

export const Label: React.FC<LabelProps> = ({ className, ...props }) => {
  return <AriaLabel {...props} className={labelStyle({ className })} />;
};

// Description
const descriptionStyle = tv({
  base: "text-sm text-muted-foreground",
});

export interface DescriptionProps extends AriaTextProps {
  className?: string;
}

export const Description: React.FC<DescriptionProps> = ({
  className,
  ...props
}) => {
  return (
    <AriaText
      {...props}
      slot="description"
      className={descriptionStyle({ className })}
    />
  );
};

// FieldError
const fieldErrorStyle = tv({
  base: "text-sm text-destructive",
});

export interface FieldErrorProps extends AriaFieldErrorProps {
  className?: string;
}

export const FieldError: React.FC<FieldErrorProps> = ({
  className,
  ...props
}) => {
  return (
    <AriaFieldError {...props} className={fieldErrorStyle({ className })} />
  );
};

// Field (wrapper)
const fieldStyle = tv({
  base: "flex flex-col gap-2",
});

export interface FieldProps extends GroupProps {
  className?: string;
}

export const Field: React.FC<FieldProps> = ({ className, ...props }) => {
  return (
    <Group
      {...props}
      className={composeRenderProps(className, (cn) =>
        fieldStyle({ className: cn }),
      )}
    />
  );
};

// FieldGroup (multiple fields)
const fieldGroupStyle = tv({
  base: "space-y-6",
});

export interface FieldGroupProps extends React.HTMLAttributes<HTMLDivElement> {}

export const FieldGroup: React.FC<FieldGroupProps> = ({
  className,
  ...props
}) => {
  return <div className={fieldGroupStyle({ className })} {...props} />;
};
