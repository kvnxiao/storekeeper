import { useMutation } from "@tanstack/solid-query";
import { deepEqual } from "fast-equals";
import { createMemo, createRoot } from "solid-js";
import { createStore, produce, unwrap } from "solid-js/store";
import { queryClient } from "@/modules/core/core.queryClient";
import { saveSettingsMutationOptions } from "@/modules/settings/settings.query";
import type { AppConfig, GamesConfig, SecretsConfig } from "@/modules/settings/settings.types";

interface SettingsFormState {
  config: AppConfig | null;
  secrets: SecretsConfig | null;
  originalConfig: AppConfig | null;
  originalSecrets: SecretsConfig | null;
}

function createSettingsForm() {
  const [state, setState] = createStore<SettingsFormState>({
    config: null,
    secrets: null,
    originalConfig: null,
    originalSecrets: null,
  });

  const saveMutation = useMutation(
    () => saveSettingsMutationOptions(queryClient),
    () => queryClient,
  );

  const isDirty = createMemo(() => {
    if (!state.config || !state.secrets || !state.originalConfig || !state.originalSecrets) {
      return false;
    }
    return (
      !deepEqual(state.config, state.originalConfig) ||
      !deepEqual(state.secrets, state.originalSecrets)
    );
  });

  /**
   * Seeds the form from loaded queries; no-op once initialized.
   *
   * Clones the working copies too - the arguments are the query cache's own
   * objects, and `produce` in the updaters would otherwise mutate the cache
   * in place, leaking unsaved edits to other cache readers.
   */
  function initialize(config: AppConfig, secrets: SecretsConfig): void {
    if (state.config || state.secrets) {
      return;
    }
    setState({
      config: structuredClone(config),
      secrets: structuredClone(secrets),
      originalConfig: structuredClone(config),
      originalSecrets: structuredClone(secrets),
    });
  }

  function updateConfig<K extends keyof AppConfig>(section: K, value: AppConfig[K]): void {
    setState(
      produce((s) => {
        if (s.config) {
          s.config[section] = value;
        }
      }),
    );
  }

  function updateSecrets<K extends keyof SecretsConfig>(section: K, value: SecretsConfig[K]): void {
    setState(
      produce((s) => {
        if (s.secrets) {
          s.secrets[section] = value;
        }
      }),
    );
  }

  function updateGameConfig<K extends keyof GamesConfig>(game: K, value: GamesConfig[K]): void {
    setState(
      produce((s) => {
        if (s.config) {
          s.config.games[game] = value;
        }
      }),
    );
  }

  /** Reverts edited state back to original snapshots. */
  function reset(): void {
    const { originalConfig, originalSecrets } = unwrap(state);
    setState({
      config: structuredClone(originalConfig),
      secrets: structuredClone(originalSecrets),
    });
  }

  /** Updates original snapshots after a successful save. */
  function markAsSaved(): void {
    const { config, secrets } = unwrap(state);
    setState({
      originalConfig: structuredClone(config),
      originalSecrets: structuredClone(secrets),
    });
  }

  /** Coordinated save action - delegates to the save-settings mutation. */
  async function save(): Promise<void> {
    const { config, secrets } = unwrap(state);
    if (!config || !secrets) {
      return;
    }

    try {
      await saveMutation.mutateAsync({ config, secrets });
      markAsSaved();
    } catch {
      // Surfaced reactively via saveError.
    }
  }

  return {
    config: () => state.config,
    secrets: () => state.secrets,
    isDirty,
    saveError: () =>
      saveMutation.error ? `Failed to save settings: ${String(saveMutation.error)}` : null,
    isSaving: () => saveMutation.isPending,
    initialize,
    updateConfig,
    updateSecrets,
    updateGameConfig,
    save,
    reset,
  };
}

export const settingsForm = createRoot(createSettingsForm);
