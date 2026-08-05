import { formOptions } from "@tanstack/solid-form";
import { unwrap } from "solid-js/store";
import type { SettingsDraft } from "@/modules/settings/settings.query";

/**
 * Options for the settings form. Defaults are a per-mount snapshot of the
 * loaded config/secrets queries: cloned so form edits never alias the query
 * cache, unwrapped because structuredClone rejects store proxies. Submission
 * goes through saveSettingsMutationOptions, which writes the saved draft back
 * to the cache.
 */
export function settingsFormOptions(defaults: SettingsDraft) {
  return formOptions({
    defaultValues: {
      config: structuredClone(unwrap(defaults.config)),
      secrets: structuredClone(unwrap(defaults.secrets)),
    },
  });
}
