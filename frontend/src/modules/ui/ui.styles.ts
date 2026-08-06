import { cn } from "tailwind-variants";

/**
 * Utility for merging Tailwind class names with conflict resolution.
 * Uses tailwind-merge under the hood to properly handle class precedence.
 */
export { cn };

// Shared form-field anatomy (TextField, NumberField, Select)

export const fieldStyle = "group flex flex-col gap-1 font-sans";

export const labelStyle = "text-sm font-medium text-zinc-950 dark:text-white";

export const descriptionStyle = "text-sm text-zinc-500 dark:text-zinc-400";
