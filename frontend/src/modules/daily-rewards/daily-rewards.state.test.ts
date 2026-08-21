import { createRoot, createSignal } from "solid-js";
import { afterEach, beforeEach, describe, expect, it, vi } from "vite-plus/test";
import { refreshDailyRewardStatus } from "@/modules/daily-rewards/daily-rewards.query";
import { createDailyRewardsState } from "@/modules/daily-rewards/daily-rewards.state";

vi.mock("@/modules/daily-rewards/daily-rewards.query", () => ({
  refreshDailyRewardStatus: vi.fn(async () => {}),
}));

const refresh = vi.mocked(refreshDailyRewardStatus);

/** 23:00 UTC+8 on 2026-08-05, an hour before the reset. */
const BEFORE_RESET = Temporal.Instant.from("2026-08-05T15:00:00Z");
const AFTER_RESET = Temporal.Instant.from("2026-08-05T16:30:00Z");

/** Lets the deferred tick effect register and then run. */
async function flushEffects(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

describe("daily rewards state", () => {
  let dispose: () => void;
  let setTick: (value: Temporal.Instant) => void;

  beforeEach(() => {
    vi.useFakeTimers();
    refresh.mockClear();

    createRoot((disposeRoot) => {
      dispose = disposeRoot;
      const [tick, setTickSignal] = createSignal(BEFORE_RESET);
      setTick = setTickSignal;
      createDailyRewardsState().init(tick);
    });
  });

  afterEach(() => {
    dispose();
    vi.useRealTimers();
  });

  it("fetches claim status on init", () => {
    expect(refresh).toHaveBeenCalledTimes(1);
  });

  it("re-fetches after the UTC+8 date rolls over, once the buffer elapses", async () => {
    await flushEffects();
    refresh.mockClear();

    setTick(AFTER_RESET);
    await flushEffects();

    expect(refresh).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(60_000);
    expect(refresh).toHaveBeenCalledTimes(1);
  });

  it("ignores ticks within the same UTC+8 date", async () => {
    await flushEffects();
    refresh.mockClear();

    setTick(BEFORE_RESET.add({ minutes: 30 }));
    await flushEffects();
    await vi.advanceTimersByTimeAsync(60_000);

    expect(refresh).not.toHaveBeenCalled();
  });

  it("re-fetches only once per rollover", async () => {
    await flushEffects();
    refresh.mockClear();

    setTick(AFTER_RESET);
    await flushEffects();
    setTick(AFTER_RESET.add({ minutes: 1 }));
    await flushEffects();
    await vi.advanceTimersByTimeAsync(60_000);

    expect(refresh).toHaveBeenCalledTimes(1);
  });
});
