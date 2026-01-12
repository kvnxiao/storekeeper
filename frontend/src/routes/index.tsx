import { Cog6ToothIcon } from "@heroicons/react/24/outline";
import { createFileRoute } from "@tanstack/react-router";
import { useAtomValue } from "jotai";
import { useMemo } from "react";

import { GAME_METADATA, GAME_ORDER } from "@/modules/games/games.constants";
import type { GameId } from "@/modules/games/games.types";
import { GameSection } from "@/modules/resources/components/GameSection";
import {
  resourcesEventListenerAtom,
  resourcesQueryAtom,
} from "@/modules/resources/resources.atoms";
import { ButtonLink } from "@/modules/ui/components/ButtonLink";

const HomePage: React.FC = () => {
  const { data: resources, error } = useAtomValue(resourcesQueryAtom);

  // Subscribe to backend resource updates
  useAtomValue(resourcesEventListenerAtom);

  const activeGames = useMemo((): GameId[] => {
    if (!resources?.games) return [];
    return GAME_ORDER.filter((gameId) => resources.games[gameId] !== undefined);
  }, [resources]);

  const hasAnyResources = activeGames.length > 0;

  return (
    <div className="min-h-screen p-4">
      <header className="mb-4 flex items-center justify-between">
        <h1 className="text-xl font-bold text-zinc-950 dark:text-white">
          Storekeeper
        </h1>
        <ButtonLink
          to="/settings"
          variant="plain"
          aria-label="Settings"
          onClick={() => {
            document.documentElement.dataset.viewTransitionDirection =
              "forward";
          }}
        >
          <Cog6ToothIcon className="h-5 w-5" />
        </ButtonLink>
      </header>

      {error && (
        <div className="mb-4 rounded-lg bg-red-500/15 p-3 text-red-700 ring-1 ring-red-500/20 dark:text-red-400">
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
          <div className="py-8 text-center text-zinc-500 dark:text-zinc-400">
            <p className="mb-2">No games configured</p>
            <p className="text-sm">
              Add your game credentials in the config file to get started.
            </p>
          </div>
        )}
      </main>
    </div>
  );
};

export const Route = createFileRoute("/")({
  component: HomePage,
});
