import { mutationOptions, queryOptions } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  SaveResult,
  SecretsConfig,
} from "@/modules/settings/settings.types";

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

/** Mutation options for saving config + secrets and applying changes in one call */
export function saveAndApplyMutationOptions() {
  return mutationOptions({
    mutationKey: ["save-and-apply"],
    mutationFn: async (params: { config: AppConfig; secrets: SecretsConfig }) =>
      invoke<SaveResult>("save_and_apply", params),
  });
}
