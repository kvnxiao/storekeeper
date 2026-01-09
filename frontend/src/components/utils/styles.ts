import { tv } from "tailwind-variants";

/**
 * Focus ring styles for react-aria-components.
 * Uses the tailwindcss-react-aria-components plugin variants.
 */
export const focusRingStyle = tv({
  base: "outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
});

/**
 * Disabled state styles for components.
 */
export const disabledStyle = tv({
  base: "disabled:pointer-events-none disabled:opacity-50",
});
