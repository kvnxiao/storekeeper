import type { JSX, ParentComponent } from "solid-js";
import { ResourceIcon } from "@/modules/resources/components/ResourceIcon";
import { resourcesState } from "@/modules/resources/resources.state";
import { cn } from "@/modules/ui/ui.styles";

export interface ResourceCardProps {
  iconPath: string;
  name: string;
  hasData: boolean;
  /** Rendered opposite the name; the caller owns its loading fallback. */
  trailing: JSX.Element;
  /** `baseline` sits a numeric value on the name's text baseline. */
  align?: "baseline" | "center";
}

/**
 * Shell shared by the resource cards. The shell, icon, and name stay mounted
 * across the loading and loaded states; only the value regions swap. Swapping
 * the whole subtree when data first arrives makes every card flash at once.
 */
export const ResourceCard: ParentComponent<ResourceCardProps> = (props) => {
  return (
    <div
      class={cn(
        "rounded-lg bg-zinc-50 p-2 transition-transform hover:translate-x-0.5 dark:bg-zinc-700",
        (resourcesState.isRefreshing() || !props.hasData) && "mask-shimmer",
      )}
    >
      <div class="flex items-center gap-2">
        <ResourceIcon src={props.iconPath} />
        <div
          class={cn(
            "flex min-w-0 flex-1 justify-between gap-2",
            props.align === "baseline" ? "items-baseline" : "items-center",
          )}
        >
          <span class="truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
            {props.name}
          </span>
          {props.trailing}
        </div>
      </div>
      {props.children}
    </div>
  );
};
