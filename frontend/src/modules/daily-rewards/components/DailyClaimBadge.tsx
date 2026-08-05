import { useMutation } from "@tanstack/solid-query";
import { Match, Show, Switch, type VoidComponent } from "solid-js";
import { createClaimStatus } from "@/modules/daily-rewards/daily-rewards.primitives";
import { claimDailyRewardMutationOptions } from "@/modules/daily-rewards/daily-rewards.query";
import type { GameId } from "@/modules/games/games.types";
import { Badge } from "@/modules/ui/components/Badge";
import * as m from "@/paraglide/messages";

export interface DailyClaimBadgeProps {
  gameId: GameId;
}

/** Daily-reward claim status for a game; unclaimed is clickable to claim. */
export const DailyClaimBadge: VoidComponent<DailyClaimBadgeProps> = (props) => {
  const claimStatus = createClaimStatus(props.gameId);
  const claim = useMutation(() => claimDailyRewardMutationOptions());

  return (
    <Show when={claimStatus() != null}>
      <Switch
        fallback={
          // `relative` lifts the button above GameSection's stretched trigger
          // overlay so the claim click does not toggle the disclosure.
          <button
            type="button"
            class="relative cursor-pointer rounded-md outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onClick={() => claim.mutate(props.gameId)}
          >
            <Badge variant="warning">{m.daily_unclaimed()}</Badge>
          </button>
        }
      >
        <Match when={claim.isPending}>
          <Badge variant="default">{m.daily_claiming()}</Badge>
        </Match>
        <Match when={claimStatus() === true}>
          <Badge variant="success">{m.daily_claimed()}</Badge>
        </Match>
      </Switch>
    </Show>
  );
};
