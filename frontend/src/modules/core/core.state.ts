import { invoke } from "@tauri-apps/api/core";
import { type EventCallback, type EventName, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { createMemo, createRoot, createSignal, onCleanup } from "solid-js";
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

export function createCore() {
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

  const timeWithSecondsFormatter = createMemo(
    () =>
      new Intl.DateTimeFormat(locale(), {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
        hour12: false,
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

  let shellInitialized = false;
  let dashboardInitialized = false;
  let localeTimeout: ReturnType<typeof setTimeout> | undefined;
  const unlisteners: Promise<UnlistenFn>[] = [];

  function subscribe<T>(name: EventName, handler: EventCallback<T>): void {
    unlisteners.push(listen<T>(name, handler));
  }

  // Because the init functions run from a component's onMount scope, that scope
  // can dispose while the factory root remains alive. Register cleanup in the
  // factory root and keep the init flags in its closure so component disposal
  // does not register listeners again.
  onCleanup(() => {
    clearInterval(tickInterval);
    clearTimeout(localeTimeout);
    for (const pending of unlisteners) {
      pending.then((unlisten) => unlisten()).catch(console.error);
    }
  });

  /**
   * Starts what every window needs: the tick interval and the resolved locale.
   *
   * Every Tauri window runs its own module registry, so this runs once per
   * window. It does not register a backend listener or reach a game API, so a
   * secondary window boots without repeating the dashboard's work.
   */
  function initShell(): void {
    if (shellInitialized) {
      return;
    }
    shellInitialized = true;

    startTickInterval();

    // Sync Paraglide locale from backend config on startup. Views wait on
    // localeReady so the first render already uses the resolved locale. The
    // timeout is what keeps a command that never settles from leaving the
    // window permanently blank; a rejection releases the views the same way.
    localeTimeout = setTimeout(() => setLocaleReady(true), LOCALE_RESOLVE_TIMEOUT_MS);
    void invoke<string>("get_effective_locale")
      .then((effectiveLocale) => setAppLocale(effectiveLocale))
      .catch(console.error)
      .finally(() => {
        clearTimeout(localeTimeout);
        setLocaleReady(true);
      });
  }

  /**
   * Registers the backend listeners and seeds the daily-reward state.
   *
   * `app_handle.emit` broadcasts to every webview, so only the dashboard window
   * calls this; a second registration would double-handle every event and issue
   * a second round of live game-API requests.
   */
  function initDashboard(): void {
    if (dashboardInitialized) {
      return;
    }
    dashboardInitialized = true;

    subscribe(AppEvent.RefreshStarted, () => {
      resourcesState.refreshStarted();
    });

    subscribe<AllResources>(AppEvent.ResourcesUpdated, (event) => {
      resourcesState.refreshSettled();
      queryClient.setQueryData(resourcesQueryOptions().queryKey, event.payload);
      refreshTick();
    });

    subscribe<GameResourcePayload>(AppEvent.GameResourceUpdated, (event) => {
      const { gameId, data } = event.payload;
      resourcesState.gameSettled(gameId);
      queryClient.setQueryData(
        resourcesQueryOptions().queryKey,
        (old: AllResources | undefined) => ({
          ...old,
          games: { ...old?.games, [gameId]: data },
        }),
      );
      refreshTick();
    });

    subscribe(AppEvent.DailyRewardClaimed, () => {
      invalidateDailyRewardStatus().catch(console.error);
    });

    subscribe(AppEvent.DailyRewardStatusUpdated, () => {
      invalidateDailyRewardStatus().catch(console.error);
    });

    dailyRewardsState.init(tick);
  }

  return {
    locale,
    localeReady,
    durationFormatter,
    timeOnlyFormatter,
    weekdayTimeFormatter,
    timeWithSecondsFormatter,
    tick,
    setAppLocale,
    initShell,
    initDashboard,
  };
}

export const core = createRoot(createCore);
