import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createMemo, createRoot, createSignal } from "solid-js";
import { queryClient } from "@/modules/core/core.queryClient";
import type { GameId } from "@/modules/games/games.types";
import type { AllResources, GameResourcePayload } from "@/modules/resources/resources.types";
import "@formatjs/intl-durationformat/polyfill.js";
import { getLocale, isLocale, setLocale } from "@/paraglide/runtime";

// =============================================================================
// Backend response types (private, only used for extraction)
// =============================================================================

interface AllDailyRewardStatus {
  games?: Record<string, { info?: { is_signed?: boolean } }>;
  lastChecked?: string;
}

function extractClaimStatus(status: AllDailyRewardStatus): Map<GameId, boolean> {
  const map = new Map<GameId, boolean>();
  if (status.games) {
    for (const [gameId, data] of Object.entries(status.games)) {
      if (data.info?.is_signed != null) {
        map.set(gameId as GameId, data.info.is_signed);
      }
    }
  }
  return map;
}

// =============================================================================
// Constants
// =============================================================================

/** UTC+8 offset in milliseconds (all HoYoLab games reset at midnight UTC+8). */
const UTC8_OFFSET_MS = 8 * 3_600_000;

function getUtc8DateString(): string {
  return new Date(Date.now() + UTC8_OFFSET_MS).toISOString().slice(0, 10);
}

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
  // Refresh state - tracks when a manual refresh is in progress
  // ---------------------------------------------------------------------------

  const [isRefreshing, setIsRefreshing] = createSignal(false);

  // ---------------------------------------------------------------------------
  // Daily reward claim status - tracks whether today's reward has been claimed
  // ---------------------------------------------------------------------------

  const [dailyClaimStatus, setDailyClaimStatus] = createSignal<ReadonlyMap<GameId, boolean>>(
    new Map(),
  );

  async function fetchDailyRewardStatus(
    command: "refresh_daily_reward_status" | "get_daily_reward_status",
  ): Promise<void> {
    const status = await invoke<AllDailyRewardStatus>(command);
    setDailyClaimStatus(extractClaimStatus(status));
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

    void listen("refresh-started", () => {
      setIsRefreshing(true);
    });

    void listen<AllResources>("resources-updated", (event) => {
      setIsRefreshing(false);
      queryClient.setQueryData(["resources"], event.payload);
      refreshTick();
    });

    void listen<GameResourcePayload>("game-resource-updated", (event) => {
      const { gameId, data } = event.payload;
      queryClient.setQueryData<AllResources>(["resources"], (old) => ({
        ...old,
        games: { ...old?.games, [gameId]: data },
      }));
      refreshTick();
    });

    void listen("daily-reward-claimed", () => {
      fetchDailyRewardStatus("get_daily_reward_status").catch(console.error);
    });

    fetchDailyRewardStatus("refresh_daily_reward_status").catch(console.error);

    // Daily reset watcher: detect UTC+8 date change and re-fetch claim status
    let lastUtc8Date = getUtc8DateString();
    setInterval(() => {
      const currentDate = getUtc8DateString();
      if (currentDate !== lastUtc8Date) {
        lastUtc8Date = currentDate;
        // Buffer for game server reset propagation
        setTimeout(() => {
          fetchDailyRewardStatus("refresh_daily_reward_status").catch(console.error);
        }, 60_000);
      }
    }, 60_000);

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
    isRefreshing,
    dailyClaimStatus,
    refreshTick,
    setAppLocale,
    init,
  };
}

export const core = createRoot(createCore);
