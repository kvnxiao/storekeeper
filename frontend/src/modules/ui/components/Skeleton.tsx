import type { VoidComponent } from "solid-js";
import { cn } from "@/modules/ui/ui.styles";

export interface SkeletonProps {
  /** Sizing and spacing utilities; heights should match the line-height of the
   * text the placeholder stands in for (`h-5` for `text-sm`, `h-4` for
   * `text-xs`) so nothing shifts when the value arrives. */
  class: string;
}

export const Skeleton: VoidComponent<SkeletonProps> = (props) => (
  <div class={cn("rounded bg-zinc-200 dark:bg-zinc-600", props.class)} />
);
