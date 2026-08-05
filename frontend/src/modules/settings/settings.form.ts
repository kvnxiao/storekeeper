import { formOptions } from "@tanstack/solid-form";
import type { SettingsDraft } from "@/modules/settings/settings.query";

/**
 * Options for the settings form. Defaults are a per-mount snapshot of the
 * loaded config/secrets queries; submission goes through
 * saveSettingsMutationOptions, which writes the saved draft back to the cache.
 */
export function settingsFormOptions(defaults: SettingsDraft) {
  return formOptions({ defaultValues: defaults });
}
