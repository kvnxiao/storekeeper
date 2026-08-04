import * as ToggleGroup from "@kobalte/core/toggle-group";
import { For, type VoidComponent } from "solid-js";
import { tv } from "tailwind-variants";

const groupStyle = tv({
  base: "inline-flex rounded-lg bg-zinc-100 p-0.5 dark:bg-zinc-800",
});

const buttonStyle = tv({
  base: [
    "cursor-default rounded-md px-2.5 py-1 text-xs font-medium text-zinc-500 outline-none transition-colors",
    "hover:bg-zinc-200/50 dark:hover:bg-zinc-700/50",
    "data-[pressed]:bg-white data-[pressed]:text-zinc-950 data-[pressed]:shadow-sm",
    "dark:text-zinc-400 dark:data-[pressed]:bg-zinc-700 dark:data-[pressed]:text-white",
  ],
});

interface SegmentedControlItem {
  id: string;
  label: string;
}

export interface SegmentedControlProps {
  "aria-label": string;
  selectedKey: string;
  onSelectionChange: (key: string) => void;
  items: SegmentedControlItem[];
}

export const SegmentedControl: VoidComponent<SegmentedControlProps> = (props) => {
  return (
    <ToggleGroup.Root
      aria-label={props["aria-label"]}
      value={props.selectedKey}
      onChange={(value) => {
        // Ignore deselection so one segment is always active
        if (value != null && value !== props.selectedKey) {
          props.onSelectionChange(value);
        }
      }}
      class={groupStyle()}
    >
      <For each={props.items}>
        {(item) => (
          <ToggleGroup.Item value={item.id} class={buttonStyle()}>
            {item.label}
          </ToggleGroup.Item>
        )}
      </For>
    </ToggleGroup.Root>
  );
};
