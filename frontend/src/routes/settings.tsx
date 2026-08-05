import { createForm } from "@tanstack/solid-form";
import { useMutation, useQuery, useQueryClient } from "@tanstack/solid-query";
import { createFileRoute } from "@tanstack/solid-router";
import ArrowLeft from "lucide-solid/icons/arrow-left";
import CircleAlert from "lucide-solid/icons/circle-alert";
import { For, Show, type VoidComponent } from "solid-js";
import { unwrap } from "solid-js/store";
import { GenshinResource, HsrResource, ZzzResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import { getResourceLimitsForGame } from "@/modules/resources/resources.utils";
import { GeneralSection } from "@/modules/settings/components/GeneralSection";
import { HoyolabGameSection } from "@/modules/settings/components/HoyolabGameSection";
import { HoyolabSecretsSection } from "@/modules/settings/components/HoyolabSecretsSection";
import { KuroSecretsSection } from "@/modules/settings/components/KuroSecretsSection";
import { WuwaSection } from "@/modules/settings/components/WuwaSection";
import { settingsFormOptions } from "@/modules/settings/settings.form";
import {
  configQueryOptions,
  saveSettingsMutationOptions,
  secretsQueryOptions,
} from "@/modules/settings/settings.query";
import type { AppConfig, HoyolabConfigKey, SecretsConfig } from "@/modules/settings/settings.types";
import { Button } from "@/modules/ui/components/Button";
import { ButtonLink } from "@/modules/ui/components/ButtonLink";
import { ErrorBanner } from "@/modules/ui/components/ErrorBanner";
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
// Settings Form
// =============================================================================

interface SettingsFormProps {
  config: AppConfig;
  secrets: SecretsConfig;
}

const SettingsForm: VoidComponent<SettingsFormProps> = (props) => {
  const queryClient = useQueryClient();
  const resourcesQuery = useQuery(() => resourcesQueryOptions());
  const save = useMutation(() => saveSettingsMutationOptions(queryClient));

  const form = createForm(() => ({
    // Snapshot the cache objects: clone so form edits never alias them,
    // unwrap because structuredClone rejects store proxies.
    ...settingsFormOptions({
      config: structuredClone(unwrap(props.config)),
      secrets: structuredClone(unwrap(props.secrets)),
    }),
    onSubmit: async ({ value, formApi }) => {
      try {
        await save.mutateAsync(value);
        formApi.reset(value);
      } catch {
        // Surfaced reactively via save.error.
      }
    },
  }));

  const isDefaultValue = form.useStore((state) => state.isDefaultValue);

  const saveError = () =>
    save.error ? m.settings_failed_to_save({ error: String(save.error) }) : null;

  const resourceLimits = (gameId: GameId) => getResourceLimitsForGame(resourcesQuery.data, gameId);

  return (
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
      <Show when={saveError()}>{(error) => <ErrorBanner class="mb-4">{error()}</ErrorBanner>}</Show>

      {/* Settings sections */}
      <div class="space-y-6">
        <form.Field name="config.general">
          {(field) => (
            <GeneralSection
              config={field().state.value}
              onChange={(general) => field().handleChange(general)}
            />
          )}
        </form.Field>

        <For each={HOYOLAB_GAMES}>
          {(game) => (
            <form.Field name={`config.games.${game.configKey}`}>
              {(field) => (
                <HoyolabGameSection
                  title={game.title()}
                  description={game.description()}
                  gameId={game.gameId}
                  resourceTypes={game.resourceTypes}
                  config={field().state.value}
                  resourceLimits={resourceLimits(game.gameId)}
                  onChange={(value) => field().handleChange(value)}
                />
              )}
            </form.Field>
          )}
        </For>

        <form.Field name="config.games.wuthering_waves">
          {(field) => (
            <WuwaSection
              config={field().state.value}
              resourceLimits={resourceLimits(GameId.WutheringWaves)}
              onChange={(wuwa) => field().handleChange(wuwa)}
            />
          )}
        </form.Field>

        <form.Field name="secrets.hoyolab">
          {(field) => (
            <HoyolabSecretsSection
              secrets={field().state.value}
              onChange={(hoyolab) => field().handleChange(hoyolab)}
            />
          )}
        </form.Field>

        <form.Field name="secrets.kuro">
          {(field) => (
            <KuroSecretsSection
              secrets={field().state.value}
              onChange={(kuro) => field().handleChange(kuro)}
            />
          )}
        </form.Field>
      </div>

      {/* Floating action bar */}
      <div
        class={cn(
          "fixed bottom-0 left-0 right-0 z-50 border-t border-zinc-950/10 bg-white/80 px-4 py-3 backdrop-blur-lg dark:border-white/10 dark:bg-zinc-900/80",
          "transition-transform duration-300 ease-out motion-reduce:transition-none",
          isDefaultValue() ? "translate-y-full" : "translate-y-0",
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

          <Button onClick={() => form.reset()} disabled={save.isPending}>
            {m.settings_undo()}
          </Button>
          <Button
            onClick={() => void form.handleSubmit()}
            disabled={save.isPending}
            isPending={save.isPending}
            color="blue"
          >
            {m.settings_save()}
          </Button>
        </div>
      </div>
    </div>
  );
};

// =============================================================================
// Settings Page Component
// =============================================================================

const SettingsPage: VoidComponent = () => {
  const configQuery = useQuery(() => configQueryOptions());
  const secretsQuery = useQuery(() => secretsQueryOptions());

  const loaded = () => {
    const config = configQuery.data;
    const secrets = secretsQuery.data;
    return config && secrets ? { config, secrets } : undefined;
  };

  const loadError = () => configQuery.error ?? secretsQuery.error;

  return (
    <Show
      when={loaded()}
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
      {(loaded) => <SettingsForm config={loaded().config} secrets={loaded().secrets} />}
    </Show>
  );
};

export const Route = createFileRoute("/settings")({
  component: SettingsPage,
});
