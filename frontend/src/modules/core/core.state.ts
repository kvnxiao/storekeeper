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

/**
 * How long the first paint waits on the backend locale before giving up and
 * rendering in whatever locale Paraglide restored.
 */
const LOCALE_RESOLVE_TIMEOUT_MS = 3_000;

function createCore() {
  const [locale, setLocaleSignal] = createSignal<string>(getLocale());

  const [localeReady, setLocaleReady] = createSignal(false);

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

  const [tickVersion, setTickVersion] = createSignal(0);

  let tickInterval: ReturnType<typeof setInterval> | undefined;

  /**
   * Current time for every time-derived view.
   *
   * Reading the version is what subscribes callers to the minute interval and
   * to snapshot arrivals; the time itself comes from the real clock. Sampling
   * it into a signal instead measures deadlines the backend computed as
   * `now + remaining` against a clock older than the fetch, so a snapshot
   * landing between minute ticks shows more time than is left.
   */
  function tick(): number {
    tickVersion();
    return Date.now();
  }

  function startTickInterval(): void {
    clearInterval(tickInterval);
    tickInterval = setInterval(() => setTickVersion((version) => version + 1), 60_000);
  }

  /** Recomputes the time-derived views now and restarts the minute interval. */
  function refreshTick(): void {
    setTickVersion((version) => version + 1);
    startTickInterval();
  }

  let initialized = false;

  /** Starts the tick and the backend listeners, which live for the app's lifetime. */
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

    // Sync Paraglide locale from backend config on startup. Views wait on
    // localeReady so the first render already uses the resolved locale. The
    // timeout is what keeps a command that never settles from leaving the
    // window permanently blank; a rejection releases the views the same way.
    const releaseOnTimeout = setTimeout(() => setLocaleReady(true), LOCALE_RESOLVE_TIMEOUT_MS);
    void invoke<string>("get_effective_locale")
      .then((effectiveLocale) => setAppLocale(effectiveLocale))
      .catch(console.error)
      .finally(() => {
        clearTimeout(releaseOnTimeout);
        setLocaleReady(true);
      });
  }

  return {
    locale,
    localeReady,
    durationFormatter,
    timeOnlyFormatter,
    weekdayTimeFormatter,
    tick,
    setAppLocale,
    init,
  };
}

export const core = createRoot(createCore);
