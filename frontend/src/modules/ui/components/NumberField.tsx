import * as NumberFieldPrimitive from "@kobalte/core/number-field";
import ChevronDown from "lucide-solid/icons/chevron-down";
import ChevronUp from "lucide-solid/icons/chevron-up";
import { Show, type VoidComponent } from "solid-js";
import { descriptionStyle, fieldStyle, labelStyle } from "@/modules/ui/ui.styles";

const groupStyle = [
  "relative flex rounded-lg",
  "bg-white dark:bg-zinc-800/50",
  "shadow-sm",
  "ring-1 ring-zinc-950/10 dark:ring-white/10",
  "focus-within:ring-2 focus-within:ring-blue-500",
  "data-[disabled]:bg-zinc-100 data-[disabled]:ring-zinc-950/5 dark:data-[disabled]:bg-zinc-900",
].join(" ");

const inputStyle = [
  "w-full min-w-0 flex-1 rounded-lg bg-transparent px-3 py-1.5 text-sm",
  "text-zinc-950 placeholder:text-zinc-500 dark:text-white dark:placeholder:text-zinc-400",
  "outline-none",
  "disabled:text-zinc-400 dark:disabled:text-zinc-500",
  "[appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none",
].join(" ");

const stepperButtonStyle = [
  "flex cursor-default items-center justify-center border-0 px-1",
  "text-zinc-500 dark:text-zinc-400",
  "hover:bg-zinc-100 dark:hover:bg-zinc-700",
  "active:bg-zinc-200 dark:active:bg-zinc-600",
  "disabled:text-zinc-300 dark:disabled:text-zinc-600",
].join(" ");

const stepperDividerStyle = "w-px bg-zinc-200 dark:bg-zinc-700";

export interface NumberFieldProps {
  label?: string;
  description?: string;
  value: number;
  onChange: (value: number) => void;
  minValue?: number;
  maxValue?: number;
  step?: number;
}

export const NumberField: VoidComponent<NumberFieldProps> = (props) => {
  return (
    <NumberFieldPrimitive.Root
      rawValue={props.value}
      // Kobalte echoes the current value at mount and NaN while the input is
      // cleared; neither is a user edit and must not reach the form.
      onRawValueChange={(value) => {
        if (!Number.isNaN(value) && value !== props.value) {
          props.onChange(value);
        }
      }}
      minValue={props.minValue}
      maxValue={props.maxValue}
      step={props.step}
      class={fieldStyle}
    >
      <Show when={props.label}>
        <NumberFieldPrimitive.Label class={labelStyle}>{props.label}</NumberFieldPrimitive.Label>
      </Show>
      <div class={groupStyle}>
        <NumberFieldPrimitive.Input class={inputStyle} />
        <div class={stepperDividerStyle} />
        <div class="flex flex-col">
          <NumberFieldPrimitive.IncrementTrigger class={`${stepperButtonStyle} rounded-tr-lg`}>
            <ChevronUp aria-hidden="true" class="size-4" />
          </NumberFieldPrimitive.IncrementTrigger>
          <div class={stepperDividerStyle} />
          <NumberFieldPrimitive.DecrementTrigger class={`${stepperButtonStyle} rounded-br-lg`}>
            <ChevronDown aria-hidden="true" class="size-4" />
          </NumberFieldPrimitive.DecrementTrigger>
        </div>
      </div>
      <Show when={props.description}>
        <NumberFieldPrimitive.Description class={descriptionStyle}>
          {props.description}
        </NumberFieldPrimitive.Description>
      </Show>
    </NumberFieldPrimitive.Root>
  );
};
