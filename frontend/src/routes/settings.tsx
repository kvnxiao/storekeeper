import { ArrowLeftIcon } from "@heroicons/react/24/outline";
import { createFileRoute } from "@tanstack/react-router";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import { useCallback } from "react";

import { GeneralSection } from "@/modules/settings/components/GeneralSection";
import { HoyolabGameSection } from "@/modules/settings/components/HoyolabGameSection";
import { HoyolabSecretsSection } from "@/modules/settings/components/HoyolabSecretsSection";
import { KuroSecretsSection } from "@/modules/settings/components/KuroSecretsSection";
import { NotificationSection } from "@/modules/settings/components/NotificationSection";
import { WuwaSection } from "@/modules/settings/components/WuwaSection";
import {
  configQueryAtom,
  editedConfigAtom,
  editedSecretsAtom,
  formInitAtom,
  isDirtyAtom,
  saveAtom,
  saveConfigMutationAtom,
  saveErrorAtom,
  saveSecretsMutationAtom,
  secretsQueryAtom,
} from "@/modules/settings/settings.atoms";
import type {
  AppConfig,
  GenshinConfig,
  HsrConfig,
  SecretsConfig,
  WuwaConfig,
  ZzzConfig,
} from "@/modules/settings/settings.types";
import { Button } from "@/modules/ui/components/Button";
import { ButtonLink } from "@/modules/ui/components/ButtonLink";

// =============================================================================
// Settings Page Component
// =============================================================================

const SettingsPage: React.FC = () => {
  // Subscribe to form initialization effect (runs once when queries complete)
  useAtomValue(formInitAtom);

  // Query error state (for loading UI)
  const { error: configError } = useAtomValue(configQueryAtom);
  const { error: secretsError } = useAtomValue(secretsQueryAtom);

  // Edited state atoms
  const [config, setConfig] = useAtom(editedConfigAtom);
  const [secrets, setSecrets] = useAtom(editedSecretsAtom);
  const isDirty = useAtomValue(isDirtyAtom);

  // Save action and error state
  const saveSettings = useSetAtom(saveAtom);
  const saveError = useAtomValue(saveErrorAtom);

  // Mutations (for isPending state only)
  const { isPending: isSavingConfig } = useAtomValue(saveConfigMutationAtom);
  const { isPending: isSavingSecrets } = useAtomValue(saveSecretsMutationAtom);

  const isSaving = isSavingConfig || isSavingSecrets;

  // Helper to update nested config values
  const updateConfig = useCallback(
    <K extends keyof AppConfig>(section: K, value: AppConfig[K]) => {
      setConfig((prev) => (prev ? { ...prev, [section]: value } : prev));
    },
    [setConfig],
  );

  const updateSecrets = useCallback(
    <K extends keyof SecretsConfig>(section: K, value: SecretsConfig[K]) => {
      setSecrets((prev) => (prev ? { ...prev, [section]: value } : prev));
    },
    [setSecrets],
  );

  // Loading state
  const loadError = configError || secretsError;
  if (!config || !secrets) {
    return (
      <div className="flex min-h-screen items-center justify-center p-4">
        {loadError ? (
          <p className="text-red-500">{`Failed to load settings: ${String(loadError)}`}</p>
        ) : (
          <p className="text-zinc-500 dark:text-zinc-400">
            Loading settings...
          </p>
        )}
      </div>
    );
  }

  return (
    <div className="min-h-screen p-4">
      {/* Header */}
      <header className="mb-6 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <ButtonLink
            to="/"
            variant="plain"
            aria-label="Back"
            onClick={() => {
              document.documentElement.dataset.viewTransitionDirection = "back";
            }}
          >
            <ArrowLeftIcon className="h-5 w-5" />
          </ButtonLink>
          <h1 className="text-xl font-bold text-zinc-950 dark:text-white">
            Settings
          </h1>
        </div>
        <Button
          onPress={() => void saveSettings()}
          isDisabled={!isDirty || isSaving}
          color="blue"
        >
          {isSaving ? "Saving..." : "Save Changes"}
        </Button>
      </header>

      {/* Error display */}
      {saveError && (
        <div className="mb-4 rounded-lg bg-red-500/15 p-3 text-red-700 ring-1 ring-red-500/20 dark:text-red-400">
          {saveError}
        </div>
      )}

      {/* Dirty indicator */}
      {isDirty && (
        <div className="mb-4 rounded-lg bg-amber-500/15 p-3 text-amber-700 ring-1 ring-amber-500/20 dark:text-amber-400">
          You have unsaved changes. Click "Save Changes" to apply them.
        </div>
      )}

      {/* Settings sections */}
      <div className="space-y-6">
        <GeneralSection
          config={config.general}
          onChange={(general) => updateConfig("general", general)}
        />

        <NotificationSection
          config={config.notifications}
          onChange={(notifications) =>
            updateConfig("notifications", notifications)
          }
        />

        <HoyolabGameSection
          title="Genshin Impact"
          description="Configure your Genshin Impact account."
          config={config.games.genshin_impact}
          onChange={(genshin) =>
            updateConfig("games", {
              ...config.games,
              genshin_impact: genshin as GenshinConfig,
            })
          }
        />

        <HoyolabGameSection
          title="Honkai: Star Rail"
          description="Configure your Honkai: Star Rail account."
          config={config.games.honkai_star_rail}
          onChange={(hsr) =>
            updateConfig("games", {
              ...config.games,
              honkai_star_rail: hsr as HsrConfig,
            })
          }
        />

        <HoyolabGameSection
          title="Zenless Zone Zero"
          description="Configure your Zenless Zone Zero account."
          config={config.games.zenless_zone_zero}
          onChange={(zzz) =>
            updateConfig("games", {
              ...config.games,
              zenless_zone_zero: zzz as ZzzConfig,
            })
          }
        />

        <WuwaSection
          config={config.games.wuthering_waves}
          onChange={(wuwa) =>
            updateConfig("games", {
              ...config.games,
              wuthering_waves: wuwa as WuwaConfig,
            })
          }
        />

        <HoyolabSecretsSection
          secrets={secrets.hoyolab}
          onChange={(hoyolab) => updateSecrets("hoyolab", hoyolab)}
        />

        <KuroSecretsSection
          secrets={secrets.kuro}
          onChange={(kuro) => updateSecrets("kuro", kuro)}
        />
      </div>
    </div>
  );
};

export const Route = createFileRoute("/settings")({
  component: SettingsPage,
});
