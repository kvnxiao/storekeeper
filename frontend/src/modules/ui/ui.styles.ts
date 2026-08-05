import { cn, tv } from "tailwind-variants";

/**
 * Utility for merging Tailwind class names with conflict resolution.
 * Uses tailwind-merge under the hood to properly handle class precedence.
 */
export { cn };

// Shared form-field anatomy (TextField, NumberField, Select)

export const fieldStyle = tv({
  base: "group flex flex-col gap-1 font-sans",
});

export const labelStyle = tv({
  base: "text-sm font-medium text-zinc-950 dark:text-white",
});

export const descriptionStyle = tv({
  base: "text-sm text-zinc-500 dark:text-zinc-400",
});
