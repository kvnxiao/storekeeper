import type { ParentComponent } from "solid-js";
import { tv } from "tailwind-variants";

const errorBannerStyle = tv({
  base: "rounded-lg bg-red-500/15 p-3 text-red-700 ring-1 ring-red-500/20 dark:text-red-400",
});

export interface ErrorBannerProps {
  class?: string;
}

export const ErrorBanner: ParentComponent<ErrorBannerProps> = (props) => (
  <div class={errorBannerStyle({ class: props.class })}>{props.children}</div>
);
