import * as Collapsible from "@kobalte/core/collapsible";
import { useMutation } from "@tanstack/solid-query";
import ChevronDown from "lucide-solid/icons/chevron-down";
import { type Component, Match, type ParentComponent, Show, Switch } from "solid-js";
import { tv } from "tailwind-variants";
import { claimDailyRewardMutationOptions } from "@/modules/daily-rewards/daily-rewards.query";
import type { GameId } from "@/modules/games/games.types";
import { Badge } from "@/modules/ui/components/Badge";
import * as m from "@/paraglide/messages";

const disclosureStyle = tv({
  base: "overflow-hidden rounded-lg bg-white shadow-sm ring-1 ring-zinc-950/5 dark:bg-zinc-800 dark:ring-white/10",
});

const triggerStyle = tv({
  base: [
    "group flex w-full cursor-pointer items-center justify-between px-3 py-2 text-left transition-colors",
    "hover:bg-zinc-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring",
    "dark:hover:bg-zinc-700",
  ],
});

interface ClaimBadgeProps {
  claimed: boolean;
  isClaiming: boolean;
  canClaim: boolean;
  onClaim: (e: MouseEvent) => void;
}

const ClaimBadge: Component<ClaimBadgeProps> = (props) => (
  <Switch fallback={<Badge variant="warning">{m.daily_unclaimed()}</Badge>}>
    <Match when={props.isClaiming}>
      <Badge variant="default">{m.daily_claiming()}</Badge>
    </Match>
    <Match when={props.claimed}>
      <Badge variant="success">{m.daily_claimed()}</Badge>
    </Match>
    <Match when={props.canClaim}>
      <Badge
        variant="warning"
        // Badge renders a styled span, not a native button; keep the ARIA role.
        // oxlint-disable-next-line jsx-a11y/prefer-tag-over-role
        role="button"
        tabIndex={0}
        class="cursor-pointer"
        onClick={(e) => props.onClaim(e)}
      >
        {m.daily_unclaimed()}
      </Badge>
    </Match>
  </Switch>
);

export interface GameSectionProps {
  title: string;
  /** Required when `claimStatus` is provided, to support manual claiming. */
  gameId?: GameId;
  claimStatus?: boolean | null;
}

export const GameSection: ParentComponent<GameSectionProps> = (props) => {
  const claim = useMutation(() => claimDailyRewardMutationOptions());

  const handleClaim = (e: MouseEvent) => {
    e.stopPropagation();
    if (claim.isPending || props.gameId == null) {
      return;
    }
    claim.mutate(props.gameId);
  };

  return (
    <Collapsible.Root defaultOpen class={disclosureStyle()}>
      <Collapsible.Trigger class={triggerStyle()}>
        <span class="flex items-center gap-2">
          <span class="text-base font-semibold text-zinc-950 dark:text-white">{props.title}</span>
          <Show when={props.claimStatus != null}>
            <ClaimBadge
              claimed={props.claimStatus === true}
              isClaiming={claim.isPending}
              canClaim={props.gameId != null}
              onClaim={handleClaim}
            />
          </Show>
        </span>
        <ChevronDown
          aria-hidden="true"
          class="size-4 text-zinc-400 transition-transform duration-250 ease-out group-data-[closed]:-rotate-90 motion-reduce:transition-none"
        />
      </Collapsible.Trigger>
      <Collapsible.Content class="overflow-clip data-[expanded]:animate-[collapsible-down_250ms_ease-out] data-[closed]:animate-[collapsible-up_250ms_ease-out]">
        <div class="flex flex-col gap-1.5 px-2 pt-1.5 pb-2">{props.children}</div>
      </Collapsible.Content>
    </Collapsible.Root>
  );
};
