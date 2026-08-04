import { deepEqual } from "fast-equals";
import { createMemo, createRoot, createSignal } from "solid-js";
import { createStore, produce, unwrap } from "solid-js/store";
import { queryClient } from "@/modules/core/core.queryClient";
import { core } from "@/modules/core/core.state";
import { saveAndApply } from "@/modules/settings/settings.query";
import type { AppConfig, SecretsConfig } from "@/modules/settings/settings.types";

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

  const [saveError, setSaveError] = createSignal<string | null>(null);
  const [isSaving, setIsSaving] = createSignal(false);

  const isDirty = createMemo(() => {
    if (!state.config || !state.secrets || !state.originalConfig || !state.originalSecrets) {
      return false;
    }
    return (
      !deepEqual(state.config, state.originalConfig) ||
      !deepEqual(state.secrets, state.originalSecrets)
    );
  });

  /** Seeds the form from loaded queries; no-op once initialized. */
  function initialize(config: AppConfig, secrets: SecretsConfig): void {
    if (state.config || state.secrets) {
      return;
    }
    setState({
      config,
      secrets,
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

  /** Reverts edited state back to original snapshots. */
  function reset(): void {
    const { originalConfig, originalSecrets } = unwrap(state);
    setState(
      produce((s) => {
        if (originalConfig) {
          s.config = structuredClone(originalConfig);
        }
        if (originalSecrets) {
          s.secrets = structuredClone(originalSecrets);
        }
      }),
    );
  }

  /** Updates original snapshots after a successful save. */
  function markAsSaved(): void {
    const { config, secrets } = unwrap(state);
    setState(
      produce((s) => {
        if (config) {
          s.originalConfig = structuredClone(config);
        }
        if (secrets) {
          s.originalSecrets = structuredClone(secrets);
        }
      }),
    );
  }

  /** Coordinated save action — single IPC: write + diff + apply. */
  async function save(): Promise<void> {
    const { config, secrets } = unwrap(state);
    if (!config || !secrets) {
      return;
    }

    setSaveError(null);
    setIsSaving(true);
    try {
      const result = await saveAndApply({ config, secrets });
      queryClient.setQueryData(["config"], structuredClone(config));
      markAsSaved();
      await core.setAppLocale(result.effective_locale);
    } catch (e) {
      setSaveError(`Failed to save settings: ${String(e)}`);
    } finally {
      setIsSaving(false);
    }
  }

  return {
    config: () => state.config,
    secrets: () => state.secrets,
    isDirty,
    saveError,
    isSaving,
    initialize,
    updateConfig,
    updateSecrets,
    save,
    reset,
  };
}

export const settingsForm = createRoot(createSettingsForm);
