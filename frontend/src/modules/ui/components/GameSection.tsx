import * as Collapsible from "@kobalte/core/collapsible";
import ChevronDown from "lucide-solid/icons/chevron-down";
import type { JSX, ParentComponent } from "solid-js";
import { tv } from "tailwind-variants";
import { uiState } from "@/modules/ui/ui.state";

const disclosureStyle = tv({
  base: "group/section overflow-hidden rounded-lg bg-white shadow-sm ring-1 ring-zinc-950/5 dark:bg-zinc-800 dark:ring-white/10",
});

const headerStyle = tv({
  base: "relative flex w-full cursor-pointer items-center gap-2 px-3 py-2 transition-colors hover:bg-zinc-50 dark:hover:bg-zinc-700",
});

// The trigger button holds only the title; its ::after overlay stretches the
// click target across the header row. Interactive badges stay valid siblings
// (a button must not contain interactive content) and opt out of toggling by
// stacking above the overlay with `position: relative`.
const triggerStyle = tv({
  base: [
    "cursor-pointer text-left outline-none",
    "after:absolute after:inset-0 after:rounded-lg",
    "focus-visible:after:ring-2 focus-visible:after:ring-ring focus-visible:after:ring-inset",
  ],
});

export interface GameSectionProps {
  /** Keys the expanded state, which outlives the route the section is on. */
  sectionId: string;
  title: string;
  /** Status badge rendered beside the title, outside the collapse trigger. */
  badge?: JSX.Element;
}

export const GameSection: ParentComponent<GameSectionProps> = (props) => (
  <Collapsible.Root
    open={uiState.isSectionExpanded(props.sectionId)}
    onOpenChange={(expanded) => uiState.setSectionExpanded(props.sectionId, expanded)}
    class={disclosureStyle()}
  >
    <div class={headerStyle()}>
      <Collapsible.Trigger class={triggerStyle()}>
        <span class="text-base font-semibold text-zinc-950 dark:text-white">{props.title}</span>
      </Collapsible.Trigger>
      {props.badge}
      {/* `pointer-events-none` because the collapsed rotation gives the icon a
          stacking context, which paints it above the trigger's overlay and
          swallows clicks on the only affordance that looks clickable. */}
      <ChevronDown
        aria-hidden="true"
        class="pointer-events-none ml-auto size-4 text-zinc-400 transition-transform duration-250 ease-out group-data-[closed]/section:-rotate-90 motion-reduce:transition-none"
      />
    </div>
    <Collapsible.Content class="overflow-clip data-[expanded]:animate-[collapsible-down_250ms_ease-out] data-[closed]:animate-[collapsible-up_250ms_ease-out]">
      <div class="flex flex-col gap-1.5 px-2 pt-1.5 pb-2">{props.children}</div>
    </Collapsible.Content>
  </Collapsible.Root>
);
