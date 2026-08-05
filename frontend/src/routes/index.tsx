import { useMutation, useQuery, useQueryClient } from "@tanstack/solid-query";
import { createFileRoute } from "@tanstack/solid-router";
import RefreshClockwise from "lucide-solid/icons/refresh-cw";
import Settings from "lucide-solid/icons/settings";
import { createMemo, Show, type VoidComponent } from "solid-js";
import { GameId } from "@/modules/games/games.types";
import { GenshinSection } from "@/modules/games/genshin/components/GenshinSection";
import { HsrSection } from "@/modules/games/hsr/components/HsrSection";
import { WuwaSection } from "@/modules/games/wuwa/components/WuwaSection";
import { ZzzSection } from "@/modules/games/zzz/components/ZzzSection";
import { createIsRefreshing } from "@/modules/resources/resources.primitives";
import {
  refreshResourcesMutationOptions,
  resourcesQueryOptions,
} from "@/modules/resources/resources.query";
import { configQueryOptions } from "@/modules/settings/settings.query";
import { enabledGamesFromConfig } from "@/modules/settings/settings.utils";
import { Button } from "@/modules/ui/components/Button";
import { ButtonLink } from "@/modules/ui/components/ButtonLink";
import { ErrorBanner } from "@/modules/ui/components/ErrorBanner";
import { cn } from "@/modules/ui/ui.styles";
import { setViewTransitionDirection } from "@/modules/ui/ui.utils";
import * as m from "@/paraglide/messages";

const DashboardPage: VoidComponent = () => {
  const queryClient = useQueryClient();
  const resourcesQuery = useQuery(() => resourcesQueryOptions());
  const configQuery = useQuery(() => configQueryOptions());
  const refresh = useMutation(() => refreshResourcesMutationOptions(queryClient));
  const isRefreshing = createIsRefreshing();

  const enabledGames = createMemo(() => enabledGamesFromConfig(configQuery.data));

  return (
    <div class="mx-auto min-h-screen max-w-sm p-3">
      <header class="mb-3 flex items-center justify-between">
        <h1 class="text-lg font-bold text-zinc-950 dark:text-white">{m.app_title()}</h1>
        <div class="flex items-center gap-1">
          <Button
            variant="plain"
            aria-label={m.dashboard_refresh_resources()}
            disabled={isRefreshing()}
            onClick={() => refresh.mutate()}
          >
            <RefreshClockwise
              aria-hidden="true"
              class={cn("size-5", isRefreshing() && "animate-spin")}
            />
          </Button>
          <ButtonLink
            to="/settings"
            variant="plain"
            aria-label={m.dashboard_settings()}
            onClick={() => setViewTransitionDirection("forward")}
          >
            <Settings aria-hidden="true" class="size-5" />
          </ButtonLink>
        </div>
      </header>

      <Show when={resourcesQuery.error}>
        {(error) => <ErrorBanner class="mb-3">{String(error())}</ErrorBanner>}
      </Show>

      <main class="space-y-2">
        <Show when={!configQuery.isPending}>
          <Show
            when={enabledGames().size > 0}
            fallback={
              <div class="animate-fade-in py-8 text-center text-zinc-500 dark:text-zinc-400">
                <p class="mb-2">{m.dashboard_no_games()}</p>
                <p class="text-sm">{m.dashboard_no_games_hint()}</p>
              </div>
            }
          >
            <div class="animate-fade-in space-y-2">
              <Show when={enabledGames().has(GameId.GenshinImpact)}>
                <GenshinSection />
              </Show>
              <Show when={enabledGames().has(GameId.HonkaiStarRail)}>
                <HsrSection />
              </Show>
              <Show when={enabledGames().has(GameId.ZenlessZoneZero)}>
                <ZzzSection />
              </Show>
              <Show when={enabledGames().has(GameId.WutheringWaves)}>
                <WuwaSection />
              </Show>
            </div>
          </Show>
        </Show>
      </main>
    </div>
  );
};

export const Route = createFileRoute("/")({
  component: DashboardPage,
});
