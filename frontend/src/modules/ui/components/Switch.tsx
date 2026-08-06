import * as SwitchPrimitive from "@kobalte/core/switch";
import { children, type ParentComponent, Show } from "solid-js";

// Disabled state is read off the hidden input's native `:disabled`, not
// Kobalte's `data-disabled`: a disabled <fieldset> ancestor disables the input
// without going through Kobalte's own `disabled` prop, and the settings form
// blocks edits exactly that way while a save is in flight.
const rootStyle = [
  "group relative flex items-center gap-2 text-sm font-medium transition",
  "text-zinc-950 dark:text-white",
  "has-[:disabled]:text-zinc-400 dark:has-[:disabled]:text-zinc-500",
].join(" ");

// Every background state is gated on `group-not-has-[:disabled]` so none of them
// match while the control is disabled. The disabled background then has nothing
// to outrank, which keeps it off `!important` and out of any argument with the
// order Tailwind emits these rules in. Class names are spelled out in full
// because Tailwind only generates what it can find as a literal in the source.
const trackStyle = [
  "flex h-5 w-8 shrink-0 cursor-default items-center rounded-full px-px",
  "border border-transparent shadow-inner transition duration-200 ease-in-out",
  // Focus ring (focus lands on the hidden peer input)
  "outline-none peer-focus-visible:ring-2 peer-focus-visible:ring-ring peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-background",
  // Unchecked: rest -> hover -> press
  "group-not-has-[:disabled]:bg-zinc-200 dark:group-not-has-[:disabled]:bg-zinc-600",
  "group-not-has-[:disabled]:group-hover:bg-zinc-300 dark:group-not-has-[:disabled]:group-hover:bg-zinc-500",
  "group-not-has-[:disabled]:group-active:bg-zinc-400 dark:group-not-has-[:disabled]:group-active:bg-zinc-400",
  // Checked: rest -> hover -> press
  "group-not-has-[:disabled]:group-data-[checked]:bg-zinc-700 dark:group-not-has-[:disabled]:group-data-[checked]:bg-zinc-300",
  "group-not-has-[:disabled]:group-hover:group-data-[checked]:bg-zinc-800 dark:group-not-has-[:disabled]:group-hover:group-data-[checked]:bg-zinc-200",
  "group-not-has-[:disabled]:group-active:group-data-[checked]:bg-zinc-900 dark:group-not-has-[:disabled]:group-active:group-data-[checked]:bg-zinc-100",
  // Disabled
  "group-has-[:disabled]:cursor-not-allowed",
  "group-has-[:disabled]:bg-zinc-100 dark:group-has-[:disabled]:bg-zinc-800",
].join(" ");

const handleStyle = [
  "size-4 rounded-full bg-white shadow-xs transition duration-200 ease-in-out",
  "outline outline-1 -outline-offset-1 outline-transparent",
  "dark:bg-zinc-900",
  "translate-x-0 group-data-[checked]:translate-x-3",
  "group-has-[:disabled]:bg-zinc-50 dark:group-has-[:disabled]:bg-zinc-700",
].join(" ");

export interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}

export const Switch: ParentComponent<SwitchProps> = (props) => {
  const label = children(() => props.children);

  return (
    <SwitchPrimitive.Root
      checked={props.checked}
      onChange={(checked) => props.onChange(checked)}
      disabled={props.disabled}
      class={rootStyle}
    >
      <SwitchPrimitive.Input class="peer" />
      <SwitchPrimitive.Control class={trackStyle}>
        <SwitchPrimitive.Thumb class={handleStyle} />
      </SwitchPrimitive.Control>
      <Show when={label()}>
        <SwitchPrimitive.Label>{label()}</SwitchPrimitive.Label>
      </Show>
    </SwitchPrimitive.Root>
  );
};
