import type { Key } from "react-aria-components";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";
import { tv } from "tailwind-variants";

const groupStyle = tv({
  base: "inline-flex rounded-lg bg-zinc-100 p-0.5 dark:bg-zinc-800",
});

const buttonStyle = tv({
  base: [
    "cursor-default rounded-md px-2.5 py-1 text-xs font-medium text-zinc-500 outline-none transition-colors",
    "hover:bg-zinc-200/50 dark:hover:bg-zinc-700/50",
    "selected:bg-white selected:text-zinc-950 selected:shadow-sm",
    "dark:text-zinc-400 dark:selected:bg-zinc-700 dark:selected:text-white",
  ],
});

interface SegmentedControlItem {
  id: string;
  label: string;
}

interface SegmentedControlProps {
  "aria-label": string;
  selectedKey: string;
  onSelectionChange: (key: string) => void;
  items: SegmentedControlItem[];
}

export const SegmentedControl: React.FC<SegmentedControlProps> = ({
  "aria-label": ariaLabel,
  selectedKey,
  onSelectionChange,
  items,
}) => {
  return (
    <ToggleButtonGroup
      aria-label={ariaLabel}
      selectionMode="single"
      disallowEmptySelection
      selectedKeys={new Set<Key>([selectedKey])}
      onSelectionChange={(keys) => {
        const key = [...keys][0];
        if (typeof key === "string" && key !== selectedKey) {
          onSelectionChange(key);
        }
      }}
      className={groupStyle()}
    >
      {items.map((item) => (
        <ToggleButton key={item.id} id={item.id} className={buttonStyle()}>
          {item.label}
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
};
