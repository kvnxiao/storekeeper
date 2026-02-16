import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { atom } from "jotai";
import { atomEffect } from "jotai-effect";
import { atomWithMutation, atomWithQuery } from "jotai-tanstack-query";
import { configQueryAtom } from "@/modules/core/core.config";
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
import { getLocale, isLocale, setLocale } from "@/paraglide/runtime";

// =============================================================================
// CoreAtoms Class
// =============================================================================

export class CoreAtoms {
  // ---------------------------------------------------------------------------
  // Tick system - updates every minute for real-time countdown display
  // ---------------------------------------------------------------------------

  readonly tickBase = atom<number>(Date.now());
  readonly tickRestartSignal = atom<number>(0);

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

  readonly isRefreshingBase = atom(false);

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

  readonly isConfigLoading = atom((get) => {
    const { isPending } = get(configQueryAtom);
    return isPending;
  });

  // ---------------------------------------------------------------------------
  // Enabled games - derived from config
  // ---------------------------------------------------------------------------

  readonly enabledGames = atom((get) => {
    const { data: config } = get(configQueryAtom);
    const enabled = new Set<GameId>();
    if (config?.games.genshin_impact?.enabled)
      enabled.add(GameId.GenshinImpact);
    if (config?.games.honkai_star_rail?.enabled)
      enabled.add(GameId.HonkaiStarRail);
    if (config?.games.zenless_zone_zero?.enabled)
      enabled.add(GameId.ZenlessZoneZero);
    if (config?.games.wuthering_waves?.enabled)
      enabled.add(GameId.WutheringWaves);
    return enabled;
  });

  // ---------------------------------------------------------------------------
  // Locale sync - syncs Paraglide locale from backend config on startup
  // ---------------------------------------------------------------------------

  private readonly localeSyncEffect = atomEffect((get) => {
    const { data: config } = get(configQueryAtom);
    if (!config) return;
    const language = config.general.language;
    if (isLocale(language) && language !== getLocale()) {
      setLocale(language, { reload: false });
    }
  });

  readonly localeSync = atom((get) => {
    get(this.localeSyncEffect);
  });
}
