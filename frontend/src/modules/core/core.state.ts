import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createMemo, createRoot, createSignal } from "solid-js";
import { AppEvent } from "@/modules/core/core.constants";
import { queryClient } from "@/modules/core/core.queryClient";
import { invalidateDailyRewardStatus } from "@/modules/daily-rewards/daily-rewards.query";
import { dailyRewardsState } from "@/modules/daily-rewards/daily-rewards.state";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import { resourcesState } from "@/modules/resources/resources.state";
import type { AllResources, GameResourcePayload } from "@/modules/resources/resources.types";
import { getLocale, isLocale, setLocale } from "@/paraglide/runtime";

// =============================================================================
// Core state module
// =============================================================================

function createCore() {
  // ---------------------------------------------------------------------------
  // Locale + Intl formatters
  // ---------------------------------------------------------------------------

  const [locale, setLocaleSignal] = createSignal<string>(getLocale());

  const durationFormatter = createMemo(() => {
    const loc = locale();
    return new Intl.DurationFormat(loc, {
      style: loc.startsWith("en") ? "narrow" : "short",
    });
  });

  const timeOnlyFormatter = createMemo(
    () =>
      new Intl.DateTimeFormat(locale(), {
        hour: "numeric",
        minute: "2-digit",
      }),
  );

  const weekdayTimeFormatter = createMemo(
    () =>
      new Intl.DateTimeFormat(locale(), {
        weekday: "short",
        hour: "numeric",
        minute: "2-digit",
      }),
  );

  /** Applies a backend-effective locale to Paraglide and the reactive locale. */
  async function setAppLocale(next: string): Promise<void> {
    if (!isLocale(next)) {
      return;
    }
    await setLocale(next, { reload: false });
    setLocaleSignal(next);
  }

  // ---------------------------------------------------------------------------
  // Tick system - updates every minute for real-time countdown display
  // ---------------------------------------------------------------------------

  const [tick, setTick] = createSignal<number>(Date.now());

  let tickInterval: ReturnType<typeof setInterval> | undefined;

  function startTickInterval(): void {
    clearInterval(tickInterval);
    tickInterval = setInterval(() => setTick(Date.now()), 60_000);
  }

  /** Resets the tick to now and restarts the minute interval. */
  function refreshTick(): void {
    setTick(Date.now());
    startTickInterval();
  }

  // ---------------------------------------------------------------------------
  // Initialization - backend event listeners live for the app's lifetime
  // ---------------------------------------------------------------------------

  let initialized = false;

  function init(): void {
    if (initialized) {
      return;
    }
    initialized = true;

    startTickInterval();

    void listen(AppEvent.RefreshStarted, () => {
      resourcesState.refreshStarted();
    });

    void listen<AllResources>(AppEvent.ResourcesUpdated, (event) => {
      resourcesState.refreshSettled();
      queryClient.setQueryData(resourcesQueryOptions().queryKey, event.payload);
      refreshTick();
    });

    void listen<GameResourcePayload>(AppEvent.GameResourceUpdated, (event) => {
      const { gameId, data } = event.payload;
      queryClient.setQueryData(
        resourcesQueryOptions().queryKey,
        (old: AllResources | undefined) => ({
          ...old,
          games: { ...old?.games, [gameId]: data },
        }),
      );
      refreshTick();
    });

    void listen(AppEvent.DailyRewardClaimed, () => {
      invalidateDailyRewardStatus().catch(console.error);
    });

    dailyRewardsState.init(tick);

    // Sync Paraglide locale from backend config on startup
    invoke<string>("get_effective_locale")
      .then((effectiveLocale) => setAppLocale(effectiveLocale))
      .catch(console.error);
  }

  return {
    locale,
    durationFormatter,
    timeOnlyFormatter,
    weekdayTimeFormatter,
    tick,
    setAppLocale,
    init,
  };
}

export const core = createRoot(createCore);
