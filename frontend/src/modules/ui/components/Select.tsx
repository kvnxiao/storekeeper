import * as SelectPrimitive from "@kobalte/core/select";
import Check from "lucide-solid/icons/check";
import ChevronDown from "lucide-solid/icons/chevron-down";
import { type JSX, Show } from "solid-js";
import { fieldStyle, labelStyle } from "@/modules/ui/ui.styles";

// Select trigger button
const selectTriggerStyle = [
  "flex h-9 w-full items-center justify-between rounded-lg px-3 py-2 text-sm",
  "bg-white dark:bg-zinc-800/50",
  "shadow-sm",
  "ring-1 ring-zinc-950/10 dark:ring-white/10",
  "transition-colors",
  "disabled:cursor-not-allowed disabled:opacity-50",
  "outline-none focus-visible:ring-2 focus-visible:ring-blue-500",
].join(" ");

// Select popover - Catalyst-style with backdrop blur
const selectPopoverStyle = [
  "w-(--kb-popper-anchor-width) overflow-hidden rounded-xl p-1",
  // Catalyst-style frosted glass
  "bg-white/75 backdrop-blur-xl dark:bg-zinc-800/75",
  // Shadows and ring
  "shadow-lg ring-1 ring-zinc-950/10 dark:ring-white/10",
  // Entry animation
  "data-[expanded]:animate-[fade-in_100ms_ease-out]",
].join(" ");

// Select item - Catalyst-style with blue highlight
const selectItemStyle = [
  "group flex w-full cursor-default select-none items-center gap-x-1.5 rounded-lg px-2 py-1.5 text-sm outline-none",
  "text-zinc-950 dark:text-white",
  "data-[highlighted]:bg-blue-500 data-[highlighted]:text-white",
  "data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
].join(" ");

export interface SelectOption<T extends string> {
  id: T;
  label: string;
}

export interface SelectProps<T extends string> {
  label?: string;
  value: T;
  onChange: (value: T) => void;
  options: SelectOption<T>[];
}

// Generic in the option id so config enums (log level, locale) survive the
// round trip; a `VoidComponent<Props>` annotation cannot carry a type parameter.
export const Select = <T extends string>(props: SelectProps<T>): JSX.Element => {
  return (
    <SelectPrimitive.Root<SelectOption<T>>
      options={props.options}
      optionValue="id"
      optionTextValue="label"
      value={props.options.find((option) => option.id === props.value) ?? null}
      onChange={(option) => {
        if (option) {
          props.onChange(option.id);
        }
      }}
      itemComponent={(itemProps) => (
        <SelectPrimitive.Item item={itemProps.item} class={selectItemStyle}>
          <span class="flex size-4 items-center justify-center">
            <SelectPrimitive.ItemIndicator>
              <Check
                aria-hidden="true"
                class="size-4 text-blue-500 group-data-[highlighted]:text-white"
              />
            </SelectPrimitive.ItemIndicator>
          </span>
          <SelectPrimitive.ItemLabel>{itemProps.item.rawValue.label}</SelectPrimitive.ItemLabel>
        </SelectPrimitive.Item>
      )}
      class={fieldStyle}
    >
      <Show when={props.label}>
        <SelectPrimitive.Label class={labelStyle}>{props.label}</SelectPrimitive.Label>
      </Show>
      <SelectPrimitive.Trigger class={selectTriggerStyle}>
        <SelectPrimitive.Value<SelectOption<T>> class="flex-1 truncate text-left">
          {(state) => state.selectedOption().label}
        </SelectPrimitive.Value>
        <SelectPrimitive.Icon>
          <ChevronDown aria-hidden="true" class="size-4 text-zinc-500 dark:text-zinc-400" />
        </SelectPrimitive.Icon>
      </SelectPrimitive.Trigger>
      <SelectPrimitive.Portal>
        <SelectPrimitive.Content class={selectPopoverStyle}>
          <SelectPrimitive.Listbox class="outline-none" />
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    </SelectPrimitive.Root>
  );
};
