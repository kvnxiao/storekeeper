import { type Accessor, createEffect, createRoot, getOwner, on, runWithOwner } from "solid-js";
import { refreshDailyRewardStatus } from "@/modules/daily-rewards/daily-rewards.query";
import { utc8DateString } from "@/modules/daily-rewards/daily-rewards.utils";

/** Buffer for game server reset propagation before re-fetching claim status. */
const RESET_PROPAGATION_MS = 60_000;

export function createDailyRewardsState() {
  const owner = getOwner();

  let lastUtc8Date = utc8DateString(Date.now());

  function checkReset(): void {
    const currentDate = utc8DateString(Date.now());
    if (currentDate === lastUtc8Date) {
      return;
    }
    lastUtc8Date = currentDate;
    setTimeout(() => {
      refreshDailyRewardStatus().catch(console.error);
    }, RESET_PROPAGATION_MS);
  }

  /**
   * Fetches claim status and starts watching for the UTC+8 date rollover.
   *
   * The tick accessor is injected rather than imported so the dependency runs
   * core to daily-rewards only. Watching every tick (not just the minute
   * interval) matters because backend events keep restarting that interval.
   */
  function init(tick: Accessor<number>): void {
    refreshDailyRewardStatus().catch(console.error);
    runWithOwner(owner, () => createEffect(on(tick, checkReset, { defer: true })));
  }

  return { init };
}

/** Owns the daily reset watcher; driven by the core tick via `core.init()`. */
export const dailyRewardsState = createRoot(createDailyRewardsState);
