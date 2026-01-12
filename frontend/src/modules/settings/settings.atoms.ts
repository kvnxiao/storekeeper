import { deepEqual } from "fast-equals";
import { atom } from "jotai";
import { atomEffect } from "jotai-effect";
import { atomWithMutation, atomWithQuery } from "jotai-tanstack-query";

import {
  configQueryOptions,
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
// Query Atoms
// =============================================================================

/** Fetch config from backend */
export const configQueryAtom = atomWithQuery(() => configQueryOptions());

/** Fetch secrets from backend */
export const secretsQueryAtom = atomWithQuery(() => secretsQueryOptions());

// =============================================================================
// Edited State (local form state)
// =============================================================================

/** Locally edited config (null until loaded) */
export const editedConfigAtom = atom<AppConfig | null>(null);

/** Locally edited secrets (null until loaded) */
export const editedSecretsAtom = atom<SecretsConfig | null>(null);

/** Original config snapshot for dirty checking */
export const originalConfigAtom = atom<AppConfig | null>(null);

/** Original secrets snapshot for dirty checking */
export const originalSecretsAtom = atom<SecretsConfig | null>(null);

/** Check if form has unsaved changes */
export const isDirtyAtom = atom((get) => {
  const config = get(editedConfigAtom);
  const secrets = get(editedSecretsAtom);
  const origConfig = get(originalConfigAtom);
  const origSecrets = get(originalSecretsAtom);

  if (!config || !secrets || !origConfig || !origSecrets) return false;
  return !deepEqual(config, origConfig) || !deepEqual(secrets, origSecrets);
});

// =============================================================================
// Form Initialization Effect
// =============================================================================

/** Effect that initializes form state when queries complete (runs once) */
const formInitEffect = atomEffect((get, set) => {
  const { data: loadedConfig } = get(configQueryAtom);
  const { data: loadedSecrets } = get(secretsQueryAtom);
  const currentConfig = get(editedConfigAtom);
  const currentSecrets = get(editedSecretsAtom);

  // Only initialize if queries loaded and form not already initialized
  if (loadedConfig && loadedSecrets && !currentConfig && !currentSecrets) {
    set(editedConfigAtom, loadedConfig);
    set(editedSecretsAtom, loadedSecrets);
    set(originalConfigAtom, structuredClone(loadedConfig));
    set(originalSecretsAtom, structuredClone(loadedSecrets));
  }
});

/** Subscribe to enable form auto-initialization */
export const formInitAtom = atom((get) => {
  get(formInitEffect);
});

// =============================================================================
// Mutations
// =============================================================================

/** Save config to backend */
export const saveConfigMutationAtom = atomWithMutation(() =>
  saveConfigMutationOptions(),
);

/** Save secrets to backend */
export const saveSecretsMutationAtom = atomWithMutation(() =>
  saveSecretsMutationOptions(),
);

/** Reload config in backend (applies changes) */
export const reloadConfigMutationAtom = atomWithMutation(() =>
  reloadConfigMutationOptions(),
);

// =============================================================================
// Actions
// =============================================================================

/** Update original snapshots after successful save */
export const markAsSavedAtom = atom(null, (get, set) => {
  const config = get(editedConfigAtom);
  const secrets = get(editedSecretsAtom);
  if (config) set(originalConfigAtom, structuredClone(config));
  if (secrets) set(originalSecretsAtom, structuredClone(secrets));
});

/** Error state for save operations */
export const saveErrorAtom = atom<string | null>(null);

/** Coordinated save action - saves config + secrets, reloads, marks as saved */
export const saveAtom = atom(null, async (get, set) => {
  const config = get(editedConfigAtom);
  const secrets = get(editedSecretsAtom);
  if (!config || !secrets) return;

  set(saveErrorAtom, null);

  try {
    const { mutateAsync: doSaveConfig } = get(saveConfigMutationAtom);
    const { mutateAsync: doSaveSecrets } = get(saveSecretsMutationAtom);
    const { mutateAsync: doReloadConfig } = get(reloadConfigMutationAtom);

    await Promise.all([doSaveConfig(config), doSaveSecrets(secrets)]);
    // biome-ignore lint/nursery/useAwaitThenable: mutateAsync returns Promise, type inference issue
    await doReloadConfig();
    set(markAsSavedAtom);
  } catch (e) {
    set(saveErrorAtom, `Failed to save settings: ${String(e)}`);
  }
});
