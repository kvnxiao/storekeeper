import { createSignal, onMount, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import GameSection from "./components/GameSection";

// Types matching Rust backend
interface StaminaResource {
  current: number;
  max: number;
  seconds_until_full: number | null;
  regen_rate_seconds: number;
}

interface CooldownResource {
  is_ready: boolean;
  seconds_until_ready: number | null;
}

interface ExpeditionResource {
  current_expeditions: number;
  max_expeditions: number;
  earliest_finish_seconds: number | null;
}

interface GenshinResource {
  type: "resin" | "parametric_transformer" | "realm_currency" | "expeditions";
  data: StaminaResource | CooldownResource | ExpeditionResource;
}

interface HsrResource {
  type: "trailblaze_power";
  data: StaminaResource;
}

interface ZzzResource {
  type: "battery";
  data: StaminaResource;
}

interface WuwaResource {
  type: "waveplates";
  data: StaminaResource;
}

// Game resource type union
type GameResource = GenshinResource | HsrResource | ZzzResource | WuwaResource;

// Game ID enum matching Rust GameId
type GameId =
  | "genshin_impact"
  | "honkai_star_rail"
  | "zenless_zone_zero"
  | "wuthering_waves";

// Game metadata for display
const GAME_METADATA: Record<GameId, { title: string; shortId: string }> = {
  genshin_impact: { title: "Genshin Impact", shortId: "genshin" },
  honkai_star_rail: { title: "Honkai: Star Rail", shortId: "hsr" },
  zenless_zone_zero: { title: "Zenless Zone Zero", shortId: "zzz" },
  wuthering_waves: { title: "Wuthering Waves", shortId: "wuwa" },
};

// New HashMap-based AllResources structure
interface AllResources {
  games: Record<GameId, GameResource[]>;
  last_updated?: string;
}

function App() {
  const [resources, setResources] = createSignal<AllResources>({ games: {} });
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const fetchResources = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AllResources>("get_all_resources");
      setResources(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const refreshResources = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AllResources>("refresh_resources");
      setResources(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  onMount(async () => {
    // Initial fetch
    await fetchResources();

    // Listen for resource updates from backend
    await listen<AllResources>("resources-updated", (event) => {
      setResources(event.payload);
    });
  });

  const formatLastUpdated = () => {
    const lastUpdated = resources().last_updated;
    if (!lastUpdated) return "Never";

    const date = new Date(lastUpdated);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return "Just now";
    if (diffMins === 1) return "1 minute ago";
    if (diffMins < 60) return `${diffMins} minutes ago`;

    const diffHours = Math.floor(diffMins / 60);
    if (diffHours === 1) return "1 hour ago";
    return `${diffHours} hours ago`;
  };

  const hasAnyResources = () => {
    const res = resources();
    return res.games && Object.keys(res.games).length > 0;
  };

  // Get ordered list of games that have resources
  const getActiveGames = (): GameId[] => {
    const games = resources().games;
    if (!games) return [];

    // Return games in a consistent order
    const orderedGames: GameId[] = [
      "genshin_impact",
      "honkai_star_rail",
      "zenless_zone_zero",
      "wuthering_waves",
    ];

    return orderedGames.filter((gameId) => games[gameId] !== undefined);
  };

  return (
    <div class="min-h-screen bg-gray-50 dark:bg-gray-900 p-4">
      <header class="mb-4">
        <h1 class="text-xl font-bold text-gray-900 dark:text-white">
          Storekeeper
        </h1>
      </header>

      <Show when={error()}>
        <div class="mb-4 p-3 bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-200 rounded-lg">
          {error()}
        </div>
      </Show>

      <main class="space-y-4">
        <Show
          when={hasAnyResources()}
          fallback={
            <div class="text-center py-8 text-gray-500 dark:text-gray-400">
              <p class="mb-2">No games configured</p>
              <p class="text-sm">
                Add your game credentials in the config file to get started.
              </p>
            </div>
          }
        >
          <For each={getActiveGames()}>
            {(gameId) => {
              const metadata = GAME_METADATA[gameId];
              const gameResources = resources().games[gameId];
              return (
                <Show when={gameResources}>
                  <GameSection
                    title={metadata.title}
                    gameId={metadata.shortId}
                    resources={gameResources}
                  />
                </Show>
              );
            }}
          </For>
        </Show>
      </main>

      <footer class="mt-6 pt-4 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <span class="text-sm text-gray-500 dark:text-gray-400">
          Last updated: {formatLastUpdated()}
        </span>
        <button
          onClick={refreshResources}
          disabled={loading()}
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-lg text-sm font-medium transition-colors"
        >
          {loading() ? "Refreshing..." : "Refresh"}
        </button>
      </footer>
    </div>
  );
}

export default App;
