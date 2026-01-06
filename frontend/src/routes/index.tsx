import { createFileRoute } from "@tanstack/react-router";
import { listen } from "@tauri-apps/api/event";
import { useAtomValue, useSetAtom } from "jotai";
import { useCallback, useEffect, useMemo } from "react";
import { Button } from "react-aria-components";

import { GameSection } from "@/components/GameSection";
import { queryClient } from "@/router";
import { refreshResourcesAtom, resourcesAtom } from "@/store/atoms";
import type { AllResources, GameId } from "@/types";
import { GAME_METADATA, GAME_ORDER } from "@/types";

const HomePage: React.FC = () => {
  const {
    data: resources,
    isPending,
    error,
    refetch,
  } = useAtomValue(resourcesAtom);
  const refreshMutation = useSetAtom(refreshResourcesAtom);

  // Listen for backend resource updates
  useEffect(() => {
    const unlisten = listen<AllResources>("resources-updated", (event) => {
      queryClient.setQueryData(["resources"], event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleRefresh = useCallback(async () => {
    try {
      const result = await refreshMutation();
      queryClient.setQueryData(["resources"], result);
    } catch (e) {
      console.error("Failed to refresh:", e);
      refetch();
    }
  }, [refetch, refreshMutation]);

  const activeGames = useMemo((): GameId[] => {
    if (!resources?.games) return [];
    return GAME_ORDER.filter((gameId) => resources.games[gameId] !== undefined);
  }, [resources]);

  const hasAnyResources = activeGames.length > 0;

  const formatLastUpdated = useCallback(() => {
    const lastUpdated = resources?.lastUpdated;
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
  }, [resources?.lastUpdated]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-4">
      <header className="mb-4">
        <h1 className="text-xl font-bold text-gray-900 dark:text-white">
          Storekeeper
        </h1>
      </header>

      {error && (
        <div className="mb-4 p-3 bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-200 rounded-lg">
          {String(error)}
        </div>
      )}

      <main className="space-y-4">
        {hasAnyResources ? (
          activeGames.map((gameId) => {
            const metadata = GAME_METADATA[gameId];
            const gameResources = resources?.games[gameId];
            if (!gameResources) return null;
            return (
              <GameSection
                key={gameId}
                title={metadata.title}
                gameId={gameId}
                resources={gameResources}
              />
            );
          })
        ) : (
          <div className="text-center py-8 text-gray-500 dark:text-gray-400">
            <p className="mb-2">No games configured</p>
            <p className="text-sm">
              Add your game credentials in the config file to get started.
            </p>
          </div>
        )}
      </main>

      <footer className="mt-6 pt-4 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <span className="text-sm text-gray-500 dark:text-gray-400">
          Last updated: {formatLastUpdated()}
        </span>
        <Button
          onPress={handleRefresh}
          isDisabled={isPending}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 pressed:bg-blue-800 disabled:bg-blue-400 text-white rounded-lg text-sm font-medium transition-colors cursor-pointer"
        >
          {isPending ? "Refreshing..." : "Refresh"}
        </Button>
      </footer>
    </div>
  );
};

export const Route = createFileRoute("/")({
  component: HomePage,
});
