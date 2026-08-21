import { useMutation } from "@tanstack/solid-query";
import RefreshClockwise from "lucide-solid/icons/refresh-cw";
import Settings from "lucide-solid/icons/settings";
import { onMount, type VoidComponent } from "solid-js";
import { core } from "@/modules/core/core.state";
import { DashboardContent } from "@/modules/dashboard/components/DashboardContent";
import { refreshResourcesMutationOptions } from "@/modules/resources/resources.query";
import { resourcesState } from "@/modules/resources/resources.state";
import { Button } from "@/modules/ui/components/Button";
import { ButtonLink } from "@/modules/ui/components/ButtonLink";
import { createLoadingPhase } from "@/modules/ui/ui.loading";
import { cn } from "@/modules/ui/ui.styles";
import { setViewTransitionDirection } from "@/modules/ui/ui.utils";
import * as m from "@/paraglide/messages";

export const DashboardPage: VoidComponent = () => {
  onMount(() => core.initDashboard());

  const refresh = useMutation(() => refreshResourcesMutationOptions());

  const spinning = createLoadingPhase(() => resourcesState.isRefreshing());

  return (
    <div class="mx-auto min-h-screen max-w-sm p-3">
      <header class="mb-3 flex items-center justify-between">
        <h1 class="text-lg font-bold text-zinc-950 dark:text-white">{m.app_title()}</h1>
        <div class="flex items-center gap-1">
          <Button
            variant="plain"
            aria-label={m.dashboard_refresh_resources()}
            disabled={resourcesState.isRefreshing()}
            onClick={() => refresh.mutate()}
          >
            <RefreshClockwise
              aria-hidden="true"
              class={cn("size-5", spinning() && "animate-spin")}
            />
          </Button>
          <ButtonLink
            to="/settings"
            variant="plain"
            aria-label={m.dashboard_settings()}
            onClick={() => setViewTransitionDirection("forward")}
          >
            <Settings aria-hidden="true" class="size-5" />
          </ButtonLink>
        </div>
      </header>

      <DashboardContent />
    </div>
  );
};
