import { deepEqual } from "fast-equals";
import { atom } from "jotai";
import { atomEffect } from "jotai-effect";
import { atomWithMutation, atomWithQuery } from "jotai-tanstack-query";
import { configQueryAtom } from "@/modules/core/core.config";
import {
  reloadConfigMutationOptions,
  saveConfigMutationOptions,
  saveSecretsMutationOptions,
  secretsQueryOptions,
} from "@/modules/settings/settings.query";
import type {
  AppConfig,
  SecretsConfig,
} from "@/modules/settings/settings.types";

// =============================================================================
// SettingsAtoms Class
// =============================================================================

export class SettingsAtoms {
  // ---------------------------------------------------------------------------
  // Query Atoms
  // ---------------------------------------------------------------------------

  /** Config query from core (re-exported for convenience) */
  readonly configQuery = configQueryAtom;

  /** Fetch secrets from backend */
  readonly secretsQuery = atomWithQuery(() => secretsQueryOptions());

  // ---------------------------------------------------------------------------
  // Edited State (local form state)
  // ---------------------------------------------------------------------------

  /** Locally edited config (null until loaded) */
  readonly editedConfig = atom<AppConfig | null>(null);

  /** Locally edited secrets (null until loaded) */
  readonly editedSecrets = atom<SecretsConfig | null>(null);

  /** Original config snapshot for dirty checking */
  private readonly originalConfig = atom<AppConfig | null>(null);

  /** Original secrets snapshot for dirty checking */
  private readonly originalSecrets = atom<SecretsConfig | null>(null);

  /** Check if form has unsaved changes */
  readonly isDirty = atom((get) => {
    const config = get(this.editedConfig);
    const secrets = get(this.editedSecrets);
    const origConfig = get(this.originalConfig);
    const origSecrets = get(this.originalSecrets);

    if (!config || !secrets || !origConfig || !origSecrets) return false;
    return !deepEqual(config, origConfig) || !deepEqual(secrets, origSecrets);
  });

  // ---------------------------------------------------------------------------
  // Form Initialization Effect
  // ---------------------------------------------------------------------------

  /** Effect that initializes form state when queries complete (runs once) */
  private readonly formInitEffect = atomEffect((get, set) => {
    const { data: loadedConfig } = get(this.configQuery);
    const { data: loadedSecrets } = get(this.secretsQuery);
    const currentConfig = get(this.editedConfig);
    const currentSecrets = get(this.editedSecrets);

    // Only initialize if queries loaded and form not already initialized
    if (loadedConfig && loadedSecrets && !currentConfig && !currentSecrets) {
      set(this.editedConfig, loadedConfig);
      set(this.editedSecrets, loadedSecrets);
      set(this.originalConfig, structuredClone(loadedConfig));
      set(this.originalSecrets, structuredClone(loadedSecrets));
    }
  });

  /** Subscribe to enable form auto-initialization */
  readonly formInit = atom((get) => {
    get(this.formInitEffect);
  });

  // ---------------------------------------------------------------------------
  // Mutations
  // ---------------------------------------------------------------------------

  /** Save config to backend */
  private readonly saveConfigMutation = atomWithMutation(() =>
    saveConfigMutationOptions(),
  );

  /** Save secrets to backend */
  private readonly saveSecretsMutation = atomWithMutation(() =>
    saveSecretsMutationOptions(),
  );

  /** Reload config in backend (applies changes) */
  private readonly reloadConfigMutation = atomWithMutation(() =>
    reloadConfigMutationOptions(),
  );

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  /** Update original snapshots after successful save */
  private readonly markAsSaved = atom(null, (get, set) => {
    const config = get(this.editedConfig);
    const secrets = get(this.editedSecrets);
    if (config) set(this.originalConfig, structuredClone(config));
    if (secrets) set(this.originalSecrets, structuredClone(secrets));
  });

  /** Revert edited state back to original snapshots */
  readonly reset = atom(null, (get, set) => {
    const origConfig = get(this.originalConfig);
    const origSecrets = get(this.originalSecrets);
    if (origConfig) set(this.editedConfig, structuredClone(origConfig));
    if (origSecrets) set(this.editedSecrets, structuredClone(origSecrets));
  });

  /** Error state for save operations */
  readonly saveError = atom<string | null>(null);

  /** Derived pending state from mutation atoms */
  readonly isSaving = atom((get) => {
    const { isPending: isSavingConfig } = get(this.saveConfigMutation);
    const { isPending: isSavingSecrets } = get(this.saveSecretsMutation);
    const { isPending: isReloading } = get(this.reloadConfigMutation);
    return isSavingConfig || isSavingSecrets || isReloading;
  });

  /** Coordinated save action - saves config + secrets, reloads, marks as saved */
  readonly save = atom(null, async (get, set) => {
    const config = get(this.editedConfig);
    const secrets = get(this.editedSecrets);
    if (!config || !secrets) return;

    set(this.saveError, null);

    try {
      const { mutateAsync: doSaveConfig } = get(this.saveConfigMutation);
      const { mutateAsync: doSaveSecrets } = get(this.saveSecretsMutation);
      const { mutateAsync: doReloadConfig } = get(this.reloadConfigMutation);

      await Promise.all([doSaveConfig(config), doSaveSecrets(secrets)]);
      await doReloadConfig();
      set(this.markAsSaved);
    } catch (e) {
      set(this.saveError, `Failed to save settings: ${String(e)}`);
    }
  });
}
