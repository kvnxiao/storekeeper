import type { Key } from "react-aria-components";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

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
        if (typeof key === "string") {
          onSelectionChange(key);
        }
      }}
      className="inline-flex rounded-lg bg-zinc-100 p-0.5 dark:bg-zinc-800"
    >
      {items.map((item) => (
        <ToggleButton
          key={item.id}
          id={item.id}
          className="cursor-default rounded-md px-2.5 py-1 text-xs font-medium text-zinc-500 outline-none transition-colors selected:bg-white selected:text-zinc-950 selected:shadow-sm dark:text-zinc-400 dark:selected:bg-zinc-700 dark:selected:text-white"
        >
          {item.label}
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
};
