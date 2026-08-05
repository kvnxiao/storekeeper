import * as SwitchPrimitive from "@kobalte/core/switch";
import { children, type ParentComponent, Show } from "solid-js";
import { tv } from "tailwind-variants";
import { cn } from "@/modules/ui/ui.styles";

const trackStyle = tv({
  base: [
    "flex h-5 w-8 shrink-0 cursor-default items-center rounded-full px-px",
    "border border-transparent shadow-inner transition duration-200 ease-in-out",
    // Focus ring (focus lands on the hidden peer input)
    "outline-none peer-focus-visible:ring-2 peer-focus-visible:ring-ring peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-background",
  ],
  // Disabled is a variant rather than group-data-[disabled] overrides so the
  // hover/active classes are absent entirely - no CSS-order ambiguity.
  variants: {
    disabled: {
      false: [
        // Unchecked: rest -> hover -> press
        "bg-zinc-200 dark:bg-zinc-600",
        "group-hover:bg-zinc-300 dark:group-hover:bg-zinc-500",
        "group-active:bg-zinc-400 dark:group-active:bg-zinc-400",
        // Checked: rest -> hover -> press
        "group-data-[checked]:bg-zinc-700 dark:group-data-[checked]:bg-zinc-300",
        "group-hover:group-data-[checked]:bg-zinc-800 dark:group-hover:group-data-[checked]:bg-zinc-200",
        "group-active:group-data-[checked]:bg-zinc-900 dark:group-active:group-data-[checked]:bg-zinc-100",
      ],
      true: "cursor-not-allowed bg-zinc-100 dark:bg-zinc-800",
    },
  },
  defaultVariants: {
    disabled: false,
  },
});

const handleStyle = tv({
  base: [
    "size-4 rounded-full bg-white shadow-xs transition duration-200 ease-in-out",
    "outline outline-1 -outline-offset-1 outline-transparent",
    "dark:bg-zinc-900",
    "translate-x-0 group-data-[checked]:translate-x-3",
    "group-data-[disabled]:bg-zinc-50 dark:group-data-[disabled]:bg-zinc-700",
  ],
});

export interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  class?: string;
}

export const Switch: ParentComponent<SwitchProps> = (props) => {
  const label = children(() => props.children);

  return (
    <SwitchPrimitive.Root
      checked={props.checked}
      onChange={(checked) => props.onChange(checked)}
      disabled={props.disabled}
      class={cn(
        "group relative flex items-center gap-2 text-sm font-medium transition",
        "text-zinc-950 dark:text-white",
        "data-[disabled]:text-zinc-400 dark:data-[disabled]:text-zinc-500",
        props.class,
      )}
    >
      <SwitchPrimitive.Input class="peer" />
      <SwitchPrimitive.Control class={trackStyle({ disabled: props.disabled })}>
        <SwitchPrimitive.Thumb class={handleStyle()} />
      </SwitchPrimitive.Control>
      <Show when={label()}>
        <SwitchPrimitive.Label>{label()}</SwitchPrimitive.Label>
      </Show>
    </SwitchPrimitive.Root>
  );
};
