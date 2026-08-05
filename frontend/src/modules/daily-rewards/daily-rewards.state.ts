import { type Accessor, createEffect, createRoot, getOwner, on, runWithOwner } from "solid-js";
import { refreshDailyRewardStatus } from "@/modules/daily-rewards/daily-rewards.query";

/** UTC+8 offset in milliseconds (all HoYoLab games reset at midnight UTC+8). */
const UTC8_OFFSET_MS = 8 * 3_600_000;

/** Buffer for game server reset propagation before re-fetching claim status. */
const RESET_PROPAGATION_MS = 60_000;

function getUtc8DateString(): string {
  return new Date(Date.now() + UTC8_OFFSET_MS).toISOString().slice(0, 10);
}

function createDailyRewardsState() {
  const owner = getOwner();

  let lastUtc8Date = getUtc8DateString();

  function checkReset(): void {
    const currentDate = getUtc8DateString();
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
   * core -> daily-rewards only. Watching every tick (not just the minute
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
