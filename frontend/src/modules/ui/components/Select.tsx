import * as SelectPrimitive from "@kobalte/core/select";
import Check from "lucide-solid/icons/check";
import ChevronDown from "lucide-solid/icons/chevron-down";
import { Show, type VoidComponent } from "solid-js";
import { tv } from "tailwind-variants";
import { fieldStyle, labelStyle } from "@/modules/ui/ui.styles";

// Select trigger button
const selectTriggerStyle = tv({
  base: [
    "flex h-9 w-full items-center justify-between rounded-lg px-3 py-2 text-sm",
    "bg-white dark:bg-zinc-800/50",
    "shadow-sm",
    "ring-1 ring-zinc-950/10 dark:ring-white/10",
    "transition-colors",
    "disabled:cursor-not-allowed disabled:opacity-50",
    "outline-none focus-visible:ring-2 focus-visible:ring-blue-500",
  ],
});

// Select popover - Catalyst-style with backdrop blur
const selectPopoverStyle = tv({
  base: [
    "w-(--kb-popper-anchor-width) overflow-hidden rounded-xl p-1",
    // Catalyst-style frosted glass
    "bg-white/75 backdrop-blur-xl dark:bg-zinc-800/75",
    // Shadows and ring
    "shadow-lg ring-1 ring-zinc-950/10 dark:ring-white/10",
    // Entry animation
    "data-[expanded]:animate-[fade-in_100ms_ease-out]",
  ],
});

// Select item - Catalyst-style with blue highlight
const selectItemStyle = tv({
  base: [
    "group flex w-full cursor-default select-none items-center gap-x-1.5 rounded-lg px-2 py-1.5 text-sm outline-none",
    "text-zinc-950 dark:text-white",
    "data-[highlighted]:bg-blue-500 data-[highlighted]:text-white",
    "data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
  ],
});

export interface SelectOption {
  id: string;
  label: string;
}

export interface SelectProps {
  class?: string;
  label?: string;
  placeholder?: string;
  value: string;
  onChange: (value: string) => void;
  options: SelectOption[];
}

export const Select: VoidComponent<SelectProps> = (props) => {
  return (
    <SelectPrimitive.Root<SelectOption>
      options={props.options}
      optionValue="id"
      optionTextValue="label"
      value={props.options.find((option) => option.id === props.value) ?? null}
      onChange={(option) => {
        if (option) {
          props.onChange(option.id);
        }
      }}
      placeholder={<span class="text-muted-foreground">{props.placeholder}</span>}
      itemComponent={(itemProps) => (
        <SelectPrimitive.Item item={itemProps.item} class={selectItemStyle()}>
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
      class={fieldStyle({ class: props.class })}
    >
      <Show when={props.label}>
        <SelectPrimitive.Label class={labelStyle()}>{props.label}</SelectPrimitive.Label>
      </Show>
      <SelectPrimitive.Trigger class={selectTriggerStyle()}>
        <SelectPrimitive.Value<SelectOption> class="flex-1 truncate text-left">
          {(state) => state.selectedOption().label}
        </SelectPrimitive.Value>
        <SelectPrimitive.Icon>
          <ChevronDown aria-hidden="true" class="size-4 text-zinc-500 dark:text-zinc-400" />
        </SelectPrimitive.Icon>
      </SelectPrimitive.Trigger>
      <SelectPrimitive.Portal>
        <SelectPrimitive.Content class={selectPopoverStyle()}>
          <SelectPrimitive.Listbox class="outline-none" />
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    </SelectPrimitive.Root>
  );
};
