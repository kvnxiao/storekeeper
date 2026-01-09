import { CheckIcon, ChevronDownIcon } from "@heroicons/react/16/solid";
import type React from "react";
import {
  Select as AriaSelect,
  type SelectProps as AriaSelectProps,
  SelectValue as AriaSelectValue,
  Button,
  composeRenderProps,
  ListBox,
  ListBoxItem,
  type ListBoxItemProps,
  Popover,
} from "react-aria-components";
import { tv } from "tailwind-variants";

// Select trigger button
const selectTriggerStyle = tv({
  base: "flex h-9 w-full items-center justify-between rounded-lg border border-input bg-background px-3 py-2 text-sm shadow-sm transition-colors placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50 outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
});

// Select popover
const selectPopoverStyle = tv({
  base: "w-[var(--trigger-width)] overflow-hidden rounded-lg border border-border bg-background p-1 shadow-lg entering:animate-in entering:fade-in-0 entering:zoom-in-95 exiting:animate-out exiting:fade-out-0 exiting:zoom-out-95",
});

// Select list box
const selectListBoxStyle = tv({
  base: "outline-none",
});

// Select item
const selectItemStyle = tv({
  base: "relative flex w-full cursor-default select-none items-center rounded-md px-2 py-1.5 text-sm outline-none transition-colors focused:bg-accent focused:text-accent-foreground disabled:pointer-events-none disabled:opacity-50",
});

export interface SelectProps<T extends object>
  extends Omit<AriaSelectProps<T>, "children"> {
  className?: string;
  placeholder?: string;
  children: React.ReactNode;
}

export const Select = <T extends object>({
  className,
  placeholder,
  children,
  ...props
}: SelectProps<T>): React.ReactElement => {
  return (
    <AriaSelect {...props} className={className}>
      <Button className={selectTriggerStyle()}>
        <AriaSelectValue className="placeholder:text-muted-foreground">
          {({ defaultChildren, isPlaceholder }) =>
            isPlaceholder ? (
              <span className="text-muted-foreground">{placeholder}</span>
            ) : (
              defaultChildren
            )
          }
        </AriaSelectValue>
        <ChevronDownIcon className="size-4 opacity-50" aria-hidden="true" />
      </Button>
      <Popover className={selectPopoverStyle()}>
        <ListBox className={selectListBoxStyle()}>{children}</ListBox>
      </Popover>
    </AriaSelect>
  );
};

export interface SelectItemProps extends ListBoxItemProps {
  className?: string;
}

export const SelectItem: React.FC<SelectItemProps> = ({
  className,
  children,
  ...props
}) => {
  return (
    <ListBoxItem
      {...props}
      className={composeRenderProps(className, (cn) =>
        selectItemStyle({ className: cn }),
      )}
    >
      {composeRenderProps(children, (children, { isSelected }) => (
        <>
          <span className="absolute left-2 flex size-4 items-center justify-center">
            {isSelected && <CheckIcon className="size-4" aria-hidden="true" />}
          </span>
          <span className="pl-6">{children}</span>
        </>
      ))}
    </ListBoxItem>
  );
};
