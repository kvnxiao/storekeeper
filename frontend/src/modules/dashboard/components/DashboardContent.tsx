import { useQuery } from "@tanstack/solid-query";
import { type Component, createMemo, For, Show, type VoidComponent } from "solid-js";
import { Dynamic } from "solid-js/web";
import { StaminaGameSection } from "@/modules/games/components/StaminaGameSection";
import { HsrResource, WuwaResource, ZzzResource } from "@/modules/games/games.constants";
import { GAMES } from "@/modules/games/games.registry";
import { GameId } from "@/modules/games/games.types";
import { GenshinSection } from "@/modules/games/genshin/components/GenshinSection";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import { configQueryOptions } from "@/modules/settings/settings.query";
import { enabledGamesFromConfig } from "@/modules/settings/settings.utils";
import { ErrorBanner } from "@/modules/ui/components/ErrorBanner";
import * as m from "@/paraglide/messages";

// Kept out of the registry so it does not have to import section components,
// which import the games primitives it defines.
const GAME_SECTIONS: Record<GameId, Component> = {
  [GameId.GenshinImpact]: GenshinSection,
  [GameId.HonkaiStarRail]: () => (
    <StaminaGameSection gameId={GameId.HonkaiStarRail} resourceType={HsrResource.TrailblazePower} />
  ),
  [GameId.ZenlessZoneZero]: () => (
    <StaminaGameSection gameId={GameId.ZenlessZoneZero} resourceType={ZzzResource.Battery} />
  ),
  [GameId.WutheringWaves]: () => (
    <StaminaGameSection gameId={GameId.WutheringWaves} resourceType={WuwaResource.Waveplates} />
  ),
};

/** The dashboard's data-driven body; the route owns the header around it. */
export const DashboardContent: VoidComponent = () => {
  const resourcesQuery = useQuery(() => resourcesQueryOptions());
  const configQuery = useQuery(() => configQueryOptions());

  const enabledGames = createMemo(() => enabledGamesFromConfig(configQuery.data));

  // Without the config there is no game list, so its failure has to surface as
  // an error rather than as the "no games configured" empty state.
  const loadError = () => configQuery.error ?? resourcesQuery.error;

  return (
    <>
      <Show when={loadError()}>
        {(error) => <ErrorBanner class="mb-3">{String(error())}</ErrorBanner>}
      </Show>

      <main class="space-y-2">
        <Show when={configQuery.isSuccess}>
          <Show
            when={enabledGames().size > 0}
            fallback={
              <div class="py-8 text-center text-zinc-500 dark:text-zinc-400">
                <p class="mb-2">{m.dashboard_no_games()}</p>
                <p class="text-sm">{m.dashboard_no_games_hint()}</p>
              </div>
            }
          >
            <div class="space-y-2">
              <For each={GAMES}>
                {(game) => (
                  <Show when={enabledGames().has(game.gameId)}>
                    <Dynamic component={GAME_SECTIONS[game.gameId]} />
                  </Show>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </main>
    </>
  );
};
