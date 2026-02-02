import {
  ArrowLeftIcon,
  ExclamationCircleIcon,
} from "@heroicons/react/24/outline";
import { createFileRoute } from "@tanstack/react-router";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import { AnimatePresence, motion } from "motion/react";
import { useCallback } from "react";
import { Focusable, TooltipTrigger } from "react-aria-components";
import { atoms } from "@/modules/atoms";
import { GeneralSection } from "@/modules/settings/components/GeneralSection";
import { HoyolabGameSection } from "@/modules/settings/components/HoyolabGameSection";
import { HoyolabSecretsSection } from "@/modules/settings/components/HoyolabSecretsSection";
import { KuroSecretsSection } from "@/modules/settings/components/KuroSecretsSection";
import { NotificationSection } from "@/modules/settings/components/NotificationSection";
import { WuwaSection } from "@/modules/settings/components/WuwaSection";
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
import { Tooltip } from "@/modules/ui/components/Tooltip";

// =============================================================================
// Settings Page Component
// =============================================================================

const SettingsPage: React.FC = () => {
  // Subscribe to form initialization effect (runs once when queries complete)
  useAtomValue(atoms.settings.formInit);

  // Query error state (for loading UI)
  const { error: configError } = useAtomValue(atoms.settings.configQuery);
  const { error: secretsError } = useAtomValue(atoms.settings.secretsQuery);

  // Edited state atoms
  const [config, setConfig] = useAtom(atoms.settings.editedConfig);
  const [secrets, setSecrets] = useAtom(atoms.settings.editedSecrets);
  const isDirty = useAtomValue(atoms.settings.isDirty);

  // Save action and state
  const saveSettings = useSetAtom(atoms.settings.save);
  const saveError = useAtomValue(atoms.settings.saveError);
  const isSaving = useAtomValue(atoms.settings.isSaving);

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
        <div className="flex items-center gap-2">
          <AnimatePresence>
            {isDirty && (
              <motion.div
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.8 }}
                transition={{ duration: 0.15 }}
              >
                <TooltipTrigger delay={300}>
                  <Focusable>
                    <motion.div
                      animate={{ scale: [1, 1.1, 1] }}
                      transition={{
                        duration: 2,
                        repeat: Number.POSITIVE_INFINITY,
                        ease: "easeInOut",
                      }}
                    >
                      <ExclamationCircleIcon className="h-5 w-5 text-amber-500" />
                    </motion.div>
                  </Focusable>
                  <Tooltip placement="bottom">
                    You have unsaved changes. Click "Save Changes" to apply
                    them.
                  </Tooltip>
                </TooltipTrigger>
              </motion.div>
            )}
          </AnimatePresence>
          <motion.div
            animate={{ opacity: isDirty ? 1 : 0.5 }}
            transition={{ duration: 0.15 }}
          >
            <Button
              onPress={() => void saveSettings()}
              isDisabled={!isDirty || isSaving}
              isPending={isSaving}
              color="blue"
            >
              Save Changes
            </Button>
          </motion.div>
        </div>
      </header>

      {/* Error display */}
      {saveError && (
        <div className="mb-4 rounded-lg bg-red-500/15 p-3 text-red-700 ring-1 ring-red-500/20 dark:text-red-400">
          {saveError}
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
