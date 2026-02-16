import {
  ArrowLeftIcon,
  ExclamationCircleIcon,
} from "@heroicons/react/24/outline";
import { createFileRoute } from "@tanstack/react-router";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import { AnimatePresence, motion } from "motion/react";
import { useCallback, useMemo } from "react";
import { Button as AriaButton, TooltipTrigger } from "react-aria-components";
import { atoms } from "@/modules/atoms";
import type { GameId } from "@/modules/games/games.types";
import {
  type AllResources,
  isStaminaResource,
} from "@/modules/resources/resources.types";
import { GeneralSection } from "@/modules/settings/components/GeneralSection";
import { HoyolabGameSection } from "@/modules/settings/components/HoyolabGameSection";
import { HoyolabSecretsSection } from "@/modules/settings/components/HoyolabSecretsSection";
import { KuroSecretsSection } from "@/modules/settings/components/KuroSecretsSection";
import type { ResourceLimits } from "@/modules/settings/components/NotificationResourceRow";
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

/** Extract resource limits from backend resource data for a given game */
function getResourceLimitsForGame(
  resources: AllResources | undefined,
  gameId: GameId,
): Partial<Record<string, ResourceLimits>> | undefined {
  const gameResources = resources?.games?.[gameId];
  if (!gameResources) return undefined;

  const limits: Record<string, ResourceLimits> = {};
  for (const resource of gameResources) {
    if (isStaminaResource(resource.data)) {
      limits[resource.type] = {
        maxValue: resource.data.max,
        regenRateSeconds: resource.data.regenRateSeconds,
      };
    }
  }
  return Object.keys(limits).length > 0 ? limits : undefined;
}

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

  // Save / reset actions and state
  const saveSettings = useSetAtom(atoms.settings.save);
  const resetSettings = useSetAtom(atoms.settings.reset);
  const saveError = useAtomValue(atoms.settings.saveError);
  const isSaving = useAtomValue(atoms.settings.isSaving);

  // Resource data for computing input limits
  const { data: resources } = useAtomValue(atoms.core.resourcesQuery);
  const genshinLimits = useMemo(
    () => getResourceLimitsForGame(resources, "GENSHIN_IMPACT"),
    [resources],
  );
  const hsrLimits = useMemo(
    () => getResourceLimitsForGame(resources, "HONKAI_STAR_RAIL"),
    [resources],
  );
  const zzzLimits = useMemo(
    () => getResourceLimitsForGame(resources, "ZENLESS_ZONE_ZERO"),
    [resources],
  );
  const wuwaLimits = useMemo(
    () => getResourceLimitsForGame(resources, "WUTHERING_WAVES"),
    [resources],
  );

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
    <div className="min-h-screen p-4 pb-20">
      {/* Header */}
      <header className="mb-6 flex items-center">
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

        <HoyolabGameSection
          title="Genshin Impact"
          description="Configure your Genshin Impact account."
          gameId="GENSHIN_IMPACT"
          resourceTypes={[
            "resin",
            "parametric_transformer",
            "realm_currency",
            "expeditions",
          ]}
          config={config.games.genshin_impact}
          resourceLimits={genshinLimits}
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
          gameId="HONKAI_STAR_RAIL"
          resourceTypes={["trailblaze_power"]}
          config={config.games.honkai_star_rail}
          resourceLimits={hsrLimits}
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
          gameId="ZENLESS_ZONE_ZERO"
          resourceTypes={["battery"]}
          config={config.games.zenless_zone_zero}
          resourceLimits={zzzLimits}
          onChange={(zzz) =>
            updateConfig("games", {
              ...config.games,
              zenless_zone_zero: zzz as ZzzConfig,
            })
          }
        />

        <WuwaSection
          config={config.games.wuthering_waves}
          resourceLimits={wuwaLimits}
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

      {/* Floating action bar */}
      <AnimatePresence>
        {isDirty && (
          <motion.div
            initial={{ y: "100%" }}
            animate={{ y: 0 }}
            exit={{ y: "100%" }}
            transition={{ type: "spring", damping: 25, stiffness: 300 }}
            className="fixed bottom-0 left-0 right-0 z-50 border-t border-zinc-950/10 bg-white/80 px-4 py-3 backdrop-blur-lg dark:border-white/10 dark:bg-zinc-900/80"
          >
            <div className="flex items-center gap-3">
              <TooltipTrigger delay={300}>
                <AriaButton className="flex items-center">
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
                </AriaButton>
                <Tooltip placement="top">You have unsaved changes.</Tooltip>
              </TooltipTrigger>

              <div className="flex-1" />

              <Button onPress={() => resetSettings()} isDisabled={isSaving}>
                Undo Changes
              </Button>
              <Button
                onPress={() => void saveSettings()}
                isDisabled={isSaving}
                isPending={isSaving}
                color="blue"
              >
                Save Changes
              </Button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export const Route = createFileRoute("/settings")({
  component: SettingsPage,
});
