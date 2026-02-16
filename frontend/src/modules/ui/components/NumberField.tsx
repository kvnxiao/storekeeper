import { ChevronDownIcon, ChevronUpIcon } from "@heroicons/react/20/solid";
import type React from "react";
import {
  NumberField as AriaNumberField,
  type NumberFieldProps as AriaNumberFieldProps,
  Button,
  type ButtonProps,
  composeRenderProps,
  Group,
  Input,
  Label,
  Text,
} from "react-aria-components";
import { tv, type VariantProps } from "tailwind-variants";

const fieldStyle = tv({
  base: "group flex flex-col gap-1 font-sans",
});

const labelStyle = tv({
  base: "text-sm font-medium text-zinc-950 dark:text-white",
});

const descriptionStyle = tv({
  base: "text-sm text-zinc-500 dark:text-zinc-400",
});

const groupStyle = tv({
  base: [
    "relative flex rounded-lg",
    "bg-white dark:bg-zinc-800/50",
    "shadow-sm",
    "ring-1 ring-zinc-950/10 dark:ring-white/10",
    "focus-within:ring-2 focus-within:ring-blue-500",
    "disabled:bg-zinc-100 disabled:ring-zinc-950/5 dark:disabled:bg-zinc-900",
  ],
});

const inputStyle = tv({
  base: [
    "w-full min-w-0 flex-1 rounded-lg bg-transparent px-3 py-1.5 text-sm",
    "text-zinc-950 placeholder:text-zinc-500 dark:text-white dark:placeholder:text-zinc-400",
    "outline-none",
    "disabled:text-zinc-400 dark:disabled:text-zinc-500",
    "[appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none",
  ],
});

const stepperButtonStyle = tv({
  base: [
    "flex cursor-default items-center justify-center border-0 px-1",
    "text-zinc-500 dark:text-zinc-400",
    "hover:bg-zinc-100 dark:hover:bg-zinc-700",
    "pressed:bg-zinc-200 dark:pressed:bg-zinc-600",
    "disabled:text-zinc-300 dark:disabled:text-zinc-600",
  ],
});

const stepperDividerStyle = tv({
  base: "w-px bg-zinc-200 dark:bg-zinc-700",
});

type NumberFieldStyleProps = VariantProps<typeof fieldStyle>;

export interface NumberFieldProps
  extends AriaNumberFieldProps,
    NumberFieldStyleProps {
  label?: string;
  description?: string;
  placeholder?: string;
  className?: string;
}

const StepperButton: React.FC<ButtonProps> = ({ className, ...props }) => {
  return (
    <Button
      {...props}
      className={composeRenderProps(className, (cn) =>
        stepperButtonStyle({ className: cn }),
      )}
    />
  );
};

export const NumberField: React.FC<NumberFieldProps> = ({
  label,
  description,
  placeholder,
  className,
  ...props
}) => {
  return (
    <AriaNumberField
      {...props}
      className={composeRenderProps(className, (cn) =>
        fieldStyle({ className: cn }),
      )}
    >
      {label && <Label className={labelStyle()}>{label}</Label>}
      <Group className={groupStyle()}>
        <Input className={inputStyle()} placeholder={placeholder} />
        <div className={stepperDividerStyle()} />
        <div className="flex flex-col">
          <StepperButton slot="increment" className="rounded-tr-lg">
            <ChevronUpIcon aria-hidden className="h-4 w-4" />
          </StepperButton>
          <div className={stepperDividerStyle()} />
          <StepperButton slot="decrement" className="rounded-br-lg">
            <ChevronDownIcon aria-hidden className="h-4 w-4" />
          </StepperButton>
        </div>
      </Group>
      {description && (
        <Text slot="description" className={descriptionStyle()}>
          {description}
        </Text>
      )}
    </AriaNumberField>
  );
};
