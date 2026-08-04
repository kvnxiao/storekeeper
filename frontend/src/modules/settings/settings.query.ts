import { queryOptions } from "@tanstack/solid-query";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, SaveResult, SecretsConfig } from "@/modules/settings/settings.types";

/** Query options for fetching config from backend */
export function configQueryOptions() {
  return queryOptions({
    queryKey: ["config"],
    queryFn: async () => invoke<AppConfig>("get_config"),
    staleTime: Number.POSITIVE_INFINITY,
  });
}

/** Query options for fetching secrets from backend */
export function secretsQueryOptions() {
  return queryOptions({
    queryKey: ["secrets"],
    queryFn: async () => invoke<SecretsConfig>("get_secrets"),
    staleTime: Number.POSITIVE_INFINITY,
  });
}

/** Saves config + secrets and applies changes in a single IPC call */
export async function saveAndApply(params: {
  config: AppConfig;
  secrets: SecretsConfig;
}): Promise<SaveResult> {
  return invoke<SaveResult>("save_and_apply", params);
}
