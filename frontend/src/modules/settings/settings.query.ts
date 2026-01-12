import type { MutationOptions, QueryOptions } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import type {
  AppConfig,
  SecretsConfig,
} from "@/modules/settings/settings.types";

/** Query options for fetching config from backend */
export const configQueryOptions: QueryOptions<AppConfig> = {
  queryKey: ["config"],
  queryFn: async () => invoke<AppConfig>("get_config"),
  staleTime: Number.POSITIVE_INFINITY,
};

/** Query options for fetching secrets from backend */
export const secretsQueryOptions: QueryOptions<SecretsConfig> = {
  queryKey: ["secrets"],
  queryFn: async () => invoke<SecretsConfig>("get_secrets"),
  staleTime: Number.POSITIVE_INFINITY,
};

/** Mutation options for saving config to backend */
export const saveConfigMutationOptions: MutationOptions<
  AppConfig,
  Error,
  AppConfig
> = {
  mutationKey: ["save-config"],
  mutationFn: async (config: AppConfig) => {
    await invoke("save_config", { config });
    return config;
  },
};

/** Mutation options for saving secrets to backend */
export const saveSecretsMutationOptions: MutationOptions<
  SecretsConfig,
  Error,
  SecretsConfig
> = {
  mutationKey: ["save-secrets"],
  mutationFn: async (secrets: SecretsConfig) => {
    await invoke("save_secrets", { secrets });
    return secrets;
  },
};

/** Mutation options for reloading config in backend */
export const reloadConfigMutationOptions: MutationOptions<void, Error, void> = {
  mutationKey: ["reload-config"],
  mutationFn: async (): Promise<void> => {
    await invoke("reload_config");
  },
};
