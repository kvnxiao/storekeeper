import { ArrowPathIcon, Cog6ToothIcon } from "@heroicons/react/24/outline";
import { createFileRoute } from "@tanstack/react-router";
import { useAtomValue } from "jotai";
import { AnimatePresence, motion } from "motion/react";
import { atoms } from "@/modules/atoms";
import { GenshinSection } from "@/modules/games/genshin/components/GenshinSection";
import { HsrSection } from "@/modules/games/hsr/components/HsrSection";
import { WuwaSection } from "@/modules/games/wuwa/components/WuwaSection";
import { ZzzSection } from "@/modules/games/zzz/components/ZzzSection";
import { Button } from "@/modules/ui/components/Button";
import { ButtonLink } from "@/modules/ui/components/ButtonLink";
import * as m from "@/paraglide/messages";

const DashboardPage: React.FC = () => {
  const { error } = useAtomValue(atoms.core.resourcesQuery);
  const { isPending, mutate: refresh } = useAtomValue(
    atoms.core.refreshResources,
  );
  const isConfigLoading = useAtomValue(atoms.core.isConfigLoading);
  const enabledGames = useAtomValue(atoms.core.enabledGames);

  // Subscribe to backend resource updates
  useAtomValue(atoms.core.resourcesEventListener);
  // Sync Paraglide locale from backend config on startup
  useAtomValue(atoms.core.localeSync);

  const hasGenshin = enabledGames.has("GENSHIN_IMPACT");
  const hasHsr = enabledGames.has("HONKAI_STAR_RAIL");
  const hasZzz = enabledGames.has("ZENLESS_ZONE_ZERO");
  const hasWuwa = enabledGames.has("WUTHERING_WAVES");
  const hasAnyGames = hasGenshin || hasHsr || hasZzz || hasWuwa;

  return (
    <div className="mx-auto min-h-screen max-w-sm p-3">
      <header className="mb-3 flex items-center justify-between">
        <h1 className="text-lg font-bold text-zinc-950 dark:text-white">
          {m.app_title()}
        </h1>
        <div className="flex items-center gap-1">
          <Button
            variant="plain"
            aria-label={m.dashboard_refresh_resources()}
            isDisabled={isPending}
            onPress={() => refresh()}
          >
            <ArrowPathIcon
              className={`h-5 w-5 ${isPending ? "animate-spin" : ""}`}
            />
          </Button>
          <ButtonLink
            to="/settings"
            variant="plain"
            aria-label={m.dashboard_settings()}
            onClick={() => {
              document.documentElement.dataset.viewTransitionDirection =
                "forward";
            }}
          >
            <Cog6ToothIcon className="h-5 w-5" />
          </ButtonLink>
        </div>
      </header>

      {error && (
        <div className="mb-3 rounded-lg bg-red-500/15 p-3 text-red-700 ring-1 ring-red-500/20 dark:text-red-400">
          {String(error)}
        </div>
      )}

      <main className="space-y-2">
        <AnimatePresence>
          {!isConfigLoading &&
            (hasAnyGames ? (
              <motion.div
                key="sections"
                className="space-y-2"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 0.2 }}
              >
                {hasGenshin && <GenshinSection />}
                {hasHsr && <HsrSection />}
                {hasZzz && <ZzzSection />}
                {hasWuwa && <WuwaSection />}
              </motion.div>
            ) : (
              <motion.div
                key="empty"
                className="py-8 text-center text-zinc-500 dark:text-zinc-400"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 0.2 }}
              >
                <p className="mb-2">{m.dashboard_no_games()}</p>
                <p className="text-sm">{m.dashboard_no_games_hint()}</p>
              </motion.div>
            ))}
        </AnimatePresence>
      </main>
    </div>
  );
};

export const Route = createFileRoute("/")({
  component: DashboardPage,
});
