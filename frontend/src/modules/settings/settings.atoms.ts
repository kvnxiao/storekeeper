import { deepEqual } from "fast-equals";
import { atom } from "jotai";
import { atomEffect } from "jotai-effect";
import { atomWithMutation, atomWithQuery } from "jotai-tanstack-query";
import type { CoreAtoms } from "@/modules/core/core.atoms";
import {
  saveAndApplyMutationOptions,
  secretsQueryOptions,
} from "@/modules/settings/settings.query";
import type {
  AppConfig,
  SecretsConfig,
} from "@/modules/settings/settings.types";
import { isLocale, setLocale } from "@/paraglide/runtime";

// =============================================================================
// SettingsAtoms Class
// =============================================================================

export class SettingsAtoms {
  private readonly core: CoreAtoms;

  constructor(core: CoreAtoms) {
    this.core = core;
    this.configQuery = core.configQuery;
  }
  // ---------------------------------------------------------------------------
  // Query Atoms
  // ---------------------------------------------------------------------------

  /** Config query from core (re-exported for convenience) */
  readonly configQuery;

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

  /** Save config + secrets and apply changes in a single IPC call */
  private readonly saveAndApplyMutation = atomWithMutation(() =>
    saveAndApplyMutationOptions(),
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

  /** Derived pending state from mutation atom */
  readonly isSaving = atom((get) => {
    const { isPending } = get(this.saveAndApplyMutation);
    return isPending;
  });

  /** Coordinated save action — single IPC: write + diff + apply */
  readonly save = atom(null, async (get, set) => {
    const config = get(this.editedConfig);
    const secrets = get(this.editedSecrets);
    if (!config || !secrets) return;

    set(this.saveError, null);

    try {
      const { mutateAsync: doSaveAndApply } = get(this.saveAndApplyMutation);
      const result = await doSaveAndApply({ config, secrets });
      set(this.markAsSaved);

      // Sync frontend locale from backend's effective locale
      if (isLocale(result.effective_locale)) {
        setLocale(result.effective_locale, { reload: false });
        set(this.core.locale, result.effective_locale);
      }
    } catch (e) {
      set(this.saveError, `Failed to save settings: ${String(e)}`);
    }
  });
}
