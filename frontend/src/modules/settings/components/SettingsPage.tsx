import { createForm } from "@tanstack/solid-form";
import { useMutation, useQuery, useQueryClient } from "@tanstack/solid-query";
import ArrowLeft from "lucide-solid/icons/arrow-left";
import CircleAlert from "lucide-solid/icons/circle-alert";
import { createMemo, For, onMount, Show, type VoidComponent } from "solid-js";
import { core } from "@/modules/core/core.state";
import type { ResourceType } from "@/modules/games/games.constants";
import { GAMES } from "@/modules/games/games.registry";
import type { GameId } from "@/modules/games/games.types";
import { resourcesQueryOptions } from "@/modules/resources/resources.query";
import type { ResourceLimits } from "@/modules/resources/resources.types";
import { getResourceLimitsForGame } from "@/modules/resources/resources.utils";
import { GeneralSection } from "@/modules/settings/components/GeneralSection";
import { HoyolabGameSection } from "@/modules/settings/components/HoyolabGameSection";
import { HoyolabSecretsSection } from "@/modules/settings/components/HoyolabSecretsSection";
import { LogsSection } from "@/modules/settings/components/LogsSection";
import { KuroSecretsSection } from "@/modules/settings/components/KuroSecretsSection";
import { WuwaGameSection } from "@/modules/settings/components/WuwaGameSection";
import { settingsFormOptions } from "@/modules/settings/settings.form";
import {
  configQueryOptions,
  saveSettingsMutationOptions,
  secretsQueryOptions,
} from "@/modules/settings/settings.query";
import type { AppConfig, SecretsConfig } from "@/modules/settings/settings.types";
import { Button } from "@/modules/ui/components/Button";
import { ButtonLink } from "@/modules/ui/components/ButtonLink";
import { ErrorBanner } from "@/modules/ui/components/ErrorBanner";
import { Tooltip } from "@/modules/ui/components/Tooltip";
import { cn } from "@/modules/ui/ui.styles";
import { setViewTransitionDirection } from "@/modules/ui/ui.utils";
import * as m from "@/paraglide/messages";

interface SettingsFormProps {
  config: AppConfig;
  secrets: SecretsConfig;
}

const SettingsForm: VoidComponent<SettingsFormProps> = (props) => {
  const queryClient = useQueryClient();
  const resourcesQuery = useQuery(() => resourcesQueryOptions());
  const save = useMutation(() => saveSettingsMutationOptions(queryClient));

  const form = createForm(() => ({
    ...settingsFormOptions({ config: props.config, secrets: props.secrets }),
    onSubmit: async ({ value, formApi }) => {
      try {
        await save.mutateAsync(value);
        formApi.reset(value);
      } catch {
        // Keep the mutation error in save.error for rendering.
      }
    },
  }));

  const isDefaultValue = form.useStore((state) => state.isDefaultValue);

  const saveError = () =>
    save.error ? m.settings_failed_to_save({ error: String(save.error) }) : null;

  const resourceLimits = createMemo(() => {
    const limits = new Map<GameId, Partial<Record<ResourceType, ResourceLimits>>>();
    for (const game of GAMES) {
      limits.set(game.gameId, getResourceLimitsForGame(resourcesQuery.data, game.gameId));
    }
    return limits;
  });

  return (
    <div class="min-h-screen p-4 pb-20">
      <header class="mb-6 flex items-center">
        <div class="flex items-center gap-3">
          <ButtonLink
            to="/"
            variant="plain"
            aria-label={m.settings_back()}
            onClick={() => setViewTransitionDirection("back")}
          >
            <ArrowLeft aria-hidden="true" class="size-5" />
          </ButtonLink>
          <h1 class="text-xl font-bold text-zinc-950 dark:text-white">{m.settings_title()}</h1>
        </div>
      </header>

      <Show when={saveError()}>{(error) => <ErrorBanner class="mb-4">{error()}</ErrorBanner>}</Show>

      <form
        onSubmit={(event) => {
          event.preventDefault();
          void form.handleSubmit();
        }}
      >
        {/* Disable the fieldset while saving so edits cannot be overwritten by
            formApi.reset(value). */}
        <fieldset class="min-w-0 space-y-6" disabled={save.isPending}>
          <form.Field name="config.general">
            {(field) => (
              <GeneralSection
                config={field().state.value}
                onChange={(general) => field().handleChange(general)}
              />
            )}
          </form.Field>

          <form.Field name="config.general">
            {(field) => (
              <LogsSection
                config={field().state.value}
                onChange={(general) => field().handleChange(general)}
              />
            )}
          </form.Field>

          {/* Keep separate branches so each field preserves the config type
              selected by its configKey. */}
          <For each={GAMES}>
            {(game) => (
              <Show
                when={game.provider === "hoyolab" ? game : undefined}
                fallback={
                  <form.Field name="config.games.wuthering_waves">
                    {(field) => (
                      <WuwaGameSection
                        title={game.name()}
                        description={game.description()}
                        gameId={game.gameId}
                        resourceTypes={game.resourceTypes}
                        config={field().state.value}
                        resourceLimits={resourceLimits().get(game.gameId)}
                        onChange={(wuwa) => field().handleChange(wuwa)}
                      />
                    )}
                  </form.Field>
                }
              >
                {(hoyolab) => (
                  <form.Field name={`config.games.${hoyolab().configKey}`}>
                    {(field) => (
                      <HoyolabGameSection
                        title={game.name()}
                        description={game.description()}
                        gameId={game.gameId}
                        resourceTypes={game.resourceTypes}
                        config={field().state.value}
                        resourceLimits={resourceLimits().get(game.gameId)}
                        onChange={(value) => field().handleChange(value)}
                      />
                    )}
                  </form.Field>
                )}
              </Show>
            )}
          </For>

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
        </fieldset>

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
              <span class="sr-only">{m.settings_unsaved_changes()}</span>
            </Tooltip>

            <div class="flex-1" />

            <Button onClick={() => form.reset()} disabled={save.isPending}>
              {m.settings_undo()}
            </Button>
            <Button type="submit" disabled={save.isPending} isPending={save.isPending} color="blue">
              {m.settings_save()}
            </Button>
          </div>
        </div>
      </form>
    </div>
  );
};

export const SettingsPage: VoidComponent = () => {
  onMount(() => core.initDashboard());

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
