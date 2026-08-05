import { useQuery } from "@tanstack/solid-query";
import { createFileRoute } from "@tanstack/solid-router";
import ArrowLeft from "lucide-solid/icons/arrow-left";
import CircleAlert from "lucide-solid/icons/circle-alert";
import { createEffect, For, Show, type VoidComponent } from "solid-js";
import { GenshinResource, HsrResource, ZzzResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import { getResourceLimitsForGame } from "@/modules/resources/resources.utils";
import { GeneralSection } from "@/modules/settings/components/GeneralSection";
import { HoyolabGameSection } from "@/modules/settings/components/HoyolabGameSection";
import { HoyolabSecretsSection } from "@/modules/settings/components/HoyolabSecretsSection";
import { KuroSecretsSection } from "@/modules/settings/components/KuroSecretsSection";
import { WuwaSection } from "@/modules/settings/components/WuwaSection";
import { configQueryOptions, secretsQueryOptions } from "@/modules/settings/settings.query";
import { settingsForm } from "@/modules/settings/settings.state";
import type { HoyolabConfigKey } from "@/modules/settings/settings.types";
import { Button } from "@/modules/ui/components/Button";
import { ButtonLink } from "@/modules/ui/components/ButtonLink";
import { Tooltip } from "@/modules/ui/components/Tooltip";
import { cn } from "@/modules/ui/ui.styles";
import * as m from "@/paraglide/messages";

// =============================================================================
// HoYoLab game configuration metadata
// =============================================================================

const HOYOLAB_GAMES: {
  gameId: GameId;
  configKey: HoyolabConfigKey;
  title: () => string;
  description: () => string;
  resourceTypes: readonly string[];
}[] = [
  {
    gameId: GameId.GenshinImpact,
    configKey: "genshin_impact",
    title: m.game_genshin_name,
    description: m.settings_game_configure_genshin,
    resourceTypes: Object.values(GenshinResource),
  },
  {
    gameId: GameId.HonkaiStarRail,
    configKey: "honkai_star_rail",
    title: m.game_hsr_name,
    description: m.settings_game_configure_hsr,
    resourceTypes: Object.values(HsrResource),
  },
  {
    gameId: GameId.ZenlessZoneZero,
    configKey: "zenless_zone_zero",
    title: m.game_zzz_name,
    description: m.settings_game_configure_zzz,
    resourceTypes: Object.values(ZzzResource),
  },
];

// =============================================================================
// Settings Page Component
// =============================================================================

const loadedForm = () => {
  const config = settingsForm.config();
  const secrets = settingsForm.secrets();
  return config && secrets ? { config, secrets } : undefined;
};

const SettingsPage: VoidComponent = () => {
  const configQuery = useQuery(() => configQueryOptions());
  const secretsQuery = useQuery(() => secretsQueryOptions());
  const resourcesQuery = useQuery(() => resourcesQueryOptions());

  // Seed the form once both queries have loaded (no-op if already initialized)
  createEffect(() => {
    const config = configQuery.data;
    const secrets = secretsQuery.data;
    if (config && secrets) {
      settingsForm.initialize(config, secrets);
    }
  });

  const resourceLimits = (gameId: GameId) => getResourceLimitsForGame(resourcesQuery.data, gameId);

  const loadError = () => configQuery.error ?? secretsQuery.error;

  return (
    <Show
      when={loadedForm()}
      fallback={
        <div class="flex min-h-screen items-center justify-center p-4">
          <Show
            when={loadError()}
            fallback={<p class="text-zinc-500 dark:text-zinc-400">{m.settings_loading()}</p>}
          >
            {(error) => (
              <p class="text-red-500">{m.settings_failed_to_load({ error: String(error()) })}</p>
            )}
          </Show>
        </div>
      }
    >
      {(form) => (
        <div class="min-h-screen p-4 pb-20">
          {/* Header */}
          <header class="mb-6 flex items-center">
            <div class="flex items-center gap-3">
              <ButtonLink
                to="/"
                variant="plain"
                aria-label={m.settings_back()}
                onClick={() => {
                  document.documentElement.dataset.viewTransitionDirection = "back";
                }}
              >
                <ArrowLeft aria-hidden="true" class="size-5" />
              </ButtonLink>
              <h1 class="text-xl font-bold text-zinc-950 dark:text-white">{m.settings_title()}</h1>
            </div>
          </header>

          {/* Error display */}
          <Show when={settingsForm.saveError()}>
            {(error) => (
              <div class="mb-4 rounded-lg bg-red-500/15 p-3 text-red-700 ring-1 ring-red-500/20 dark:text-red-400">
                {error()}
              </div>
            )}
          </Show>

          {/* Settings sections */}
          <div class="space-y-6">
            <GeneralSection
              config={form().config.general}
              onChange={(general) => settingsForm.updateConfig("general", general)}
            />

            <For each={HOYOLAB_GAMES}>
              {(game) => (
                <HoyolabGameSection
                  title={game.title()}
                  description={game.description()}
                  gameId={game.gameId}
                  resourceTypes={game.resourceTypes}
                  config={form().config.games[game.configKey]}
                  resourceLimits={resourceLimits(game.gameId)}
                  onChange={(value) => settingsForm.updateGameConfig(game.configKey, value)}
                />
              )}
            </For>

            <WuwaSection
              config={form().config.games.wuthering_waves}
              resourceLimits={resourceLimits(GameId.WutheringWaves)}
              onChange={(wuwa) => settingsForm.updateGameConfig("wuthering_waves", wuwa)}
            />

            <HoyolabSecretsSection
              secrets={form().secrets.hoyolab}
              onChange={(hoyolab) => settingsForm.updateSecrets("hoyolab", hoyolab)}
            />

            <KuroSecretsSection
              secrets={form().secrets.kuro}
              onChange={(kuro) => settingsForm.updateSecrets("kuro", kuro)}
            />
          </div>

          {/* Floating action bar */}
          <div
            class={cn(
              "fixed bottom-0 left-0 right-0 z-50 border-t border-zinc-950/10 bg-white/80 px-4 py-3 backdrop-blur-lg dark:border-white/10 dark:bg-zinc-900/80",
              "transition-transform duration-300 ease-out motion-reduce:transition-none",
              settingsForm.isDirty() ? "translate-y-0" : "translate-y-full",
            )}
          >
            <div class="flex items-center gap-3">
              <Tooltip content={m.settings_unsaved_changes()} triggerClass="flex items-center">
                <CircleAlert
                  aria-hidden="true"
                  class="size-5 animate-pulse text-amber-500 motion-reduce:animate-none"
                />
              </Tooltip>

              <div class="flex-1" />

              <Button onClick={() => settingsForm.reset()} disabled={settingsForm.isSaving()}>
                {m.settings_undo()}
              </Button>
              <Button
                onClick={() => void settingsForm.save()}
                disabled={settingsForm.isSaving()}
                isPending={settingsForm.isSaving()}
                color="blue"
              >
                {m.settings_save()}
              </Button>
            </div>
          </div>
        </div>
      )}
    </Show>
  );
};

export const Route = createFileRoute("/settings")({
  component: SettingsPage,
});
