import { mutationOptions, type QueryClient, queryOptions } from "@tanstack/solid-query";
import { invoke } from "@tauri-apps/api/core";
import { core } from "@/modules/core/core.state";
import type { GameId } from "@/modules/games/games.types";
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

// A type alias (not an interface) so it satisfies Tauri's InvokeArgs via an
// implicit index signature.
export type SettingsDraft = {
  config: AppConfig;
  secrets: SecretsConfig;
};

/**
 * Mutation options for saving config + secrets in a single IPC call.
 *
 * On success the saved draft becomes the cached config/secrets and the
 * backend-effective locale is applied.
 */
export function saveSettingsMutationOptions(queryClient: QueryClient) {
  return mutationOptions({
    mutationKey: ["save-settings"],
    mutationFn: async (draft: SettingsDraft) => invoke<SaveResult>("save_and_apply", draft),
    onSuccess: async (result, draft) => {
      queryClient.setQueryData(configQueryOptions().queryKey, structuredClone(draft.config));
      queryClient.setQueryData(secretsQueryOptions().queryKey, structuredClone(draft.secrets));
      await core.setAppLocale(result.effective_locale);
    },
  });
}

/** Opens the config directory in the OS file explorer. */
export function openConfigFolder(): void {
  invoke("open_config_folder").catch(console.error);
}

/** Mutation options for sending a preview notification for a resource. */
export function previewNotificationMutationOptions() {
  return mutationOptions({
    mutationKey: ["preview-notification"],
    mutationFn: async (params: { gameId: GameId; resourceType: string }) =>
      invoke("send_preview_notification", params),
    onError: (error) => console.error("Failed to send preview notification:", error),
  });
}
