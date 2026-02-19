import { CheckIcon, ChevronDownIcon } from "@heroicons/react/16/solid";
import type React from "react";
import {
  Select as AriaSelect,
  type SelectProps as AriaSelectProps,
  SelectValue as AriaSelectValue,
  Button,
  composeRenderProps,
  Label,
  ListBox,
  ListBoxItem,
  type ListBoxItemProps,
  Popover,
} from "react-aria-components";
import { tv } from "tailwind-variants";
import { cn } from "@/modules/ui/ui.styles";

// Label style (matches TextField)
const labelStyle = tv({
  base: "text-sm font-medium text-zinc-950 dark:text-white",
});

// Select trigger button
const selectTriggerStyle = tv({
  base: [
    "flex h-9 w-full items-center justify-between rounded-lg px-3 py-2 text-sm",
    "bg-white dark:bg-zinc-800/50",
    "shadow-sm",
    "ring-1 ring-zinc-950/10 dark:ring-white/10",
    "transition-colors placeholder:text-muted-foreground",
    "disabled:cursor-not-allowed disabled:opacity-50",
    "outline-none focus-visible:ring-2 focus-visible:ring-blue-500",
  ],
});

// Select popover - Catalyst-style with backdrop blur and proper transitions
const selectPopoverStyle = tv({
  base: [
    "w-[var(--trigger-width)] overflow-hidden rounded-xl p-1",
    // Catalyst-style frosted glass
    "bg-white/75 backdrop-blur-xl dark:bg-zinc-800/75",
    // Shadows and ring
    "shadow-lg ring-1 ring-zinc-950/10 dark:ring-white/10",
    // Transitions with proper duration for exit
    "transition duration-100 ease-out",
    "entering:animate-in entering:fade-in-0 entering:zoom-in-95",
    "exiting:animate-out exiting:fade-out-0 exiting:zoom-out-95 exiting:duration-100 exiting:ease-in",
  ],
});

// Select list box
const selectListBoxStyle = tv({
  base: "outline-none",
});

// Select item - Catalyst-style with blue focus
const selectItemStyle = tv({
  base: [
    "group flex w-full cursor-default select-none items-center gap-x-1.5 rounded-lg px-2 py-1.5 text-sm outline-none",
    "text-zinc-950 dark:text-white",
    // Hover state - subtle background
    "hovered:bg-zinc-100 dark:hovered:bg-zinc-700/50",
    // Focus state - blue background like Catalyst
    "focused:bg-blue-500 focused:text-white",
    "disabled:pointer-events-none disabled:opacity-50",
  ],
});

export interface SelectProps<T extends object>
  extends Omit<AriaSelectProps<T>, "children"> {
  className?: string;
  label?: string;
  placeholder?: string;
  children: React.ReactNode;
}

export const Select = <T extends object>({
  className,
  label,
  placeholder,
  children,
  ...props
}: SelectProps<T>): React.ReactElement => {
  return (
    <AriaSelect {...props} className={cn("flex flex-col gap-1", className)}>
      {label && <Label className={labelStyle()}>{label}</Label>}
      <Button className={selectTriggerStyle()}>
        <AriaSelectValue className="flex-1 truncate text-left placeholder:text-muted-foreground">
          {({ selectedText, defaultChildren, isPlaceholder }) =>
            isPlaceholder ? (
              <span className="text-muted-foreground">{placeholder}</span>
            ) : (
              selectedText || defaultChildren
            )
          }
        </AriaSelectValue>
        <ChevronDownIcon
          className="size-4 text-zinc-500 dark:text-zinc-400"
          aria-hidden="true"
        />
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
  const textValue =
    props.textValue || (typeof children === "string" ? children : undefined);
  return (
    <ListBoxItem
      {...props}
      textValue={textValue}
      className={composeRenderProps(className, (cn) =>
        selectItemStyle({ className: cn }),
      )}
    >
      {composeRenderProps(children, (children, { isSelected }) => (
        <>
          <span className="flex size-4 items-center justify-center">
            {isSelected && (
              <CheckIcon
                className="size-4 text-blue-500 group-focused:text-white"
                aria-hidden="true"
              />
            )}
          </span>
          {children}
        </>
      ))}
    </ListBoxItem>
  );
};
