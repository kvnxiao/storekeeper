import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { atom } from "jotai";
import { atomEffect } from "jotai-effect";
import { atomWithMutation, atomWithQuery } from "jotai-tanstack-query";
import { queryClient } from "@/modules/core/core.queryClient";
import { GameId } from "@/modules/games/games.types";
import {
  refreshResourcesMutationOptions,
  resourcesQueryOptions,
} from "@/modules/resources/resources.query";
import type {
  AllResources,
  GameResourcePayload,
} from "@/modules/resources/resources.types";
import { configQueryOptions } from "@/modules/settings/settings.query";
import type { GamesConfig } from "@/modules/settings/settings.types";
import { getLocale, isLocale, setLocale } from "@/paraglide/runtime";

// =============================================================================
// Constants
// =============================================================================

export const GAME_CONFIG_KEYS: [GameId, keyof GamesConfig][] = [
  [GameId.GenshinImpact, "genshin_impact"],
  [GameId.HonkaiStarRail, "honkai_star_rail"],
  [GameId.ZenlessZoneZero, "zenless_zone_zero"],
  [GameId.WutheringWaves, "wuthering_waves"],
];

// =============================================================================
// CoreAtoms Class
// =============================================================================

export class CoreAtoms {
  // ---------------------------------------------------------------------------
  // Config Query Atom (app-wide state, shared across atom classes)
  // ---------------------------------------------------------------------------

  readonly configQuery = atomWithQuery(() => configQueryOptions());

  // ---------------------------------------------------------------------------
  // Locale atom — reactive locale for Intl formatters
  // ---------------------------------------------------------------------------

  readonly locale = atom<string>(getLocale());

  // ---------------------------------------------------------------------------
  // Tick system - updates every minute for real-time countdown display
  // ---------------------------------------------------------------------------

  private readonly tickBase = atom<number>(Date.now());
  private readonly tickRestartSignal = atom<number>(0);

  private readonly tickEffect = atomEffect((get, set) => {
    get(this.tickRestartSignal);
    set(this.tickBase, Date.now());

    const interval = setInterval(() => {
      set(this.tickBase, Date.now());
    }, 60_000);

    return () => clearInterval(interval);
  });

  readonly tick = atom((get) => {
    get(this.tickEffect);
    return get(this.tickBase);
  });

  readonly refreshTick = atom(null, (get, set) => {
    set(this.tickBase, Date.now());
    set(this.tickRestartSignal, get(this.tickRestartSignal) + 1);
  });

  // ---------------------------------------------------------------------------
  // Resources query & mutation
  // ---------------------------------------------------------------------------

  readonly resourcesQuery = atomWithQuery(() => resourcesQueryOptions());
  readonly refreshResources = atomWithMutation(() =>
    refreshResourcesMutationOptions(),
  );

  // ---------------------------------------------------------------------------
  // Refresh state - tracks when a manual refresh is in progress
  // ---------------------------------------------------------------------------

  private readonly isRefreshingBase = atom(false);

  private readonly isRefreshingEffect = atomEffect((_get, set) => {
    const unlistenPromises: Promise<UnlistenFn>[] = [];

    // Listen for refresh started
    unlistenPromises.push(
      listen("refresh-started", () => {
        set(this.isRefreshingBase, true);
      }),
    );

    // Listen for resources updated (refresh complete)
    unlistenPromises.push(
      listen("resources-updated", () => {
        set(this.isRefreshingBase, false);
      }),
    );

    return () => {
      for (const p of unlistenPromises) {
        void p.then((fn) => fn()).catch(() => {});
      }
    };
  });

  readonly isRefreshing = atom((get) => {
    get(this.isRefreshingEffect);
    return get(this.isRefreshingBase);
  });

  // ---------------------------------------------------------------------------
  // Event listener - listens for backend resource updates
  // ---------------------------------------------------------------------------

  private readonly resourcesEventEffect = atomEffect((_get, set) => {
    const unlistenPromises: Promise<UnlistenFn>[] = [];

    // Listen for full resource updates (all games at once)
    unlistenPromises.push(
      listen<AllResources>("resources-updated", (event) => {
        queryClient.setQueryData(["resources"], event.payload);
        set(this.refreshTick);
      }),
    );

    // Listen for per-game resource updates (incremental)
    unlistenPromises.push(
      listen<GameResourcePayload>("game-resource-updated", (event) => {
        const { gameId, data } = event.payload;
        queryClient.setQueryData<AllResources>(["resources"], (old) => ({
          ...old,
          games: { ...old?.games, [gameId]: data },
        }));
        set(this.refreshTick);
      }),
    );

    return () => {
      for (const p of unlistenPromises) {
        void p.then((fn) => fn()).catch(() => {});
      }
    };
  });

  readonly resourcesEventListener = atom((get) => {
    get(this.resourcesEventEffect);
  });

  // ---------------------------------------------------------------------------
  // Initial loading state - true when resources have no lastUpdated (no real data yet)
  // ---------------------------------------------------------------------------

  readonly isInitialLoading = atom((get) => {
    const { data } = get(this.resourcesQuery);
    return !data?.lastUpdated;
  });

  // ---------------------------------------------------------------------------
  // Config loading state
  // ---------------------------------------------------------------------------

  readonly isConfigLoading = atom((get) => get(this.configQuery).isPending);

  // ---------------------------------------------------------------------------
  // Enabled games - derived from config
  // ---------------------------------------------------------------------------

  readonly enabledGames = atom((get) => {
    const { data: config } = get(this.configQuery);
    return new Set<GameId>(
      GAME_CONFIG_KEYS.filter(([, key]) => config?.games[key]?.enabled).map(
        ([id]) => id,
      ),
    );
  });

  // ---------------------------------------------------------------------------
  // Locale sync - syncs Paraglide locale from backend config on startup
  // ---------------------------------------------------------------------------

  private readonly localeSyncEffect = atomEffect((_get, set) => {
    void invoke<string>("get_effective_locale").then((effectiveLocale) => {
      if (isLocale(effectiveLocale)) {
        setLocale(effectiveLocale, { reload: false });
        set(this.locale, effectiveLocale);
      }
    });
  });

  readonly localeSync = atom((get) => {
    get(this.localeSyncEffect);
  });
}
