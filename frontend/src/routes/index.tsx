import { ArrowPathIcon, Cog6ToothIcon } from "@heroicons/react/24/outline";
import { createFileRoute } from "@tanstack/react-router";
import { useAtomValue } from "jotai";

import { atoms } from "@/modules/atoms";
import { GenshinSection } from "@/modules/games/genshin/components/GenshinSection";
import { HsrSection } from "@/modules/games/hsr/components/HsrSection";
import { WuwaSection } from "@/modules/games/wuwa/components/WuwaSection";
import { ZzzSection } from "@/modules/games/zzz/components/ZzzSection";
import { Button } from "@/modules/ui/components/Button";
import { ButtonLink } from "@/modules/ui/components/ButtonLink";

const DashboardPage: React.FC = () => {
  const { data: resources, error } = useAtomValue(atoms.core.resourcesQuery);
  const { isPending, mutate: refresh } = useAtomValue(
    atoms.core.refreshResources,
  );

  // Subscribe to backend resource updates
  useAtomValue(atoms.core.resourcesEventListener);

  const hasGenshin = resources?.games?.GENSHIN_IMPACT !== undefined;
  const hasHsr = resources?.games?.HONKAI_STAR_RAIL !== undefined;
  const hasZzz = resources?.games?.ZENLESS_ZONE_ZERO !== undefined;
  const hasWuwa = resources?.games?.WUTHERING_WAVES !== undefined;
  const hasAnyResources = hasGenshin || hasHsr || hasZzz || hasWuwa;

  return (
    <div className="mx-auto min-h-screen max-w-sm p-3">
      <header className="mb-3 flex items-center justify-between">
        <h1 className="text-lg font-bold text-zinc-950 dark:text-white">
          Storekeeper
        </h1>
        <div className="flex items-center gap-1">
          <Button
            variant="plain"
            aria-label="Refresh resources"
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
            aria-label="Settings"
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
        {hasAnyResources ? (
          <>
            {hasGenshin && <GenshinSection />}
            {hasHsr && <HsrSection />}
            {hasZzz && <ZzzSection />}
            {hasWuwa && <WuwaSection />}
          </>
        ) : (
          <div className="py-8 text-center text-zinc-500 dark:text-zinc-400">
            <p className="mb-2">No games configured</p>
            <p className="text-sm">
              Add your game credentials in the config file to get started.
            </p>
          </div>
        )}
      </main>
    </div>
  );
};

export const Route = createFileRoute("/")({
  component: DashboardPage,
});
