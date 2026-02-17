# Styling Standards

## Semantic Color Tokens

Define colors as CSS custom properties using oklch format in `src/styles.css`:

```css
:root {
  --background: oklch(1 0 0);
  --foreground: oklch(0.141 0.005 285.75);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.141 0.005 285.75);
  --primary: oklch(0.21 0.006 285.75);
  --primary-foreground: oklch(0.985 0 0);
  --muted: oklch(0.967 0.001 286.38);
  --muted-foreground: oklch(0.553 0.014 285.94);
  --destructive: oklch(0.577 0.245 27.33);
}

@media (prefers-color-scheme: dark) {
  :root {
    --background: oklch(0.141 0.005 285.75);
    --foreground: oklch(0.985 0 0);
    /* ... dark variants */
  }
}
```

### Token Usage

```tsx
// Bad: Raw Tailwind colors
<div className="text-zinc-500 bg-white">

// Good: Semantic tokens
<div className="text-muted-foreground bg-background">
```

**Exception**: Component-specific styles may use zinc scale when semantic tokens don't fit.

## Class Name Merging with `cn`

Always use the `cn` utility from `tailwind-variants` (re-exported from `@/modules/ui/ui.styles`) to concatenate class strings. It uses `tailwind-merge` under the hood for proper class conflict resolution.

```tsx
import { cn } from "@/modules/ui/ui.styles";

// Bad: Template literals or string concatenation
<div className={`flex gap-2 ${className ?? ""}`}>
<div className={[baseClass, className].join(" ")}>

// Good: cn utility with automatic merge resolution
<div className={cn("flex gap-2", className)}>
```

### Why `cn` is Required

1. **Conflict resolution**: `cn("px-4", "px-2")` → `"px-2"` (later wins)
2. **Handles undefined/null**: `cn("flex", undefined, "gap-2")` → `"flex gap-2"`
3. **Consistent API**: Same pattern everywhere in the codebase

> **Exception**: When passing through a single `className` prop with no merging, use it directly — `cn()` adds no value:
>
> ```tsx
> // Unnecessary: cn() with a single argument
> <div className={cn(className)}>
>
> // Preferred: pass through directly
> <div className={className}>
> ```

### With composeRenderProps

When using React Aria's `composeRenderProps`, use `cn` for the callback:

```tsx
import { composeRenderProps } from "react-aria-components";
import { cn } from "@/modules/ui/ui.styles";

className={composeRenderProps(className, (userClassName) =>
  cn("base-styles", userClassName),
)}
```

## tailwind-variants (tv)

Use `tv()` for all component styling:

```tsx
// Bad: Inline className strings
<button className="px-4 py-2 bg-blue-500 hover:bg-blue-600 disabled:opacity-50">

// Good: Structured with tv()
const buttonStyle = tv({
  base: [
    "px-4 py-2 rounded-lg font-semibold",
    "outline-none focus-visible:ring-2 focus-visible:ring-ring",
    "disabled:opacity-50 disabled:cursor-not-allowed",
  ],
  variants: {
    variant: {
      solid: "bg-primary text-primary-foreground hover:bg-primary/90",
      outline: "border border-input bg-background hover:bg-accent",
    },
    size: {
      sm: "text-xs px-2.5 py-1",
      md: "text-sm px-3 py-1.5",
    },
  },
  defaultVariants: {
    variant: "solid",
    size: "md",
  },
});
```

### Slot-Based Styling

For multi-element components, use slots:

```tsx
const progressBarStyle = tv({
  slots: {
    root: "flex flex-col gap-1",
    track: "h-2 w-full overflow-hidden rounded-full bg-secondary",
    fill: "h-full transition-all duration-300",
    label: "text-sm font-medium",
  },
  variants: {
    color: {
      default: { fill: "bg-primary" },
      success: { fill: "bg-green-500" },
      warning: { fill: "bg-yellow-500" },
    },
  },
});

// Usage
const styles = progressBarStyle({ color: "success" });
<div className={styles.root()}>
  <div className={styles.track()}>
    <div className={styles.fill()} />
  </div>
</div>
```

## Shared Style Utilities

Create reusable style pieces with `extend`:

```tsx
// modules/ui/ui.styles.ts
export const focusRingStyle = tv({
  base: "outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
});

// Component usage
const trackStyle = tv({
  extend: focusRingStyle,
  base: ["flex h-5 w-8 items-center rounded-full"],
});
```

## Dark Mode

Dark mode is handled via `prefers-color-scheme` media query in CSS variables. Use `dark:` prefix sparingly for component-specific overrides:

```tsx
// Semantic tokens auto-switch (preferred)
<div className="bg-background text-foreground">

// dark: prefix for specific overrides
<div className="bg-white dark:bg-zinc-800">
```

## Touch Targets

Use the `touch-target` utility for interactive elements smaller than 44px:

```css
@utility touch-target {
  position: relative;
  &::before {
    content: "";
    position: absolute;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    min-height: 44px; min-width: 44px;
    @media (pointer: fine) { display: none; }
  }
}
```

## Checklist

- [ ] Use `cn()` for dynamic class concatenation (skip for single pass-through `className`)
- [ ] Use semantic color tokens over raw Tailwind colors
- [ ] Define styles with `tv()` function
- [ ] Use descriptive variant names, not booleans
- [ ] Dark mode via CSS variables, `dark:` prefix sparingly
- [ ] Extend shared utilities (focusRingStyle) where applicable
- [ ] Group base classes logically in arrays
