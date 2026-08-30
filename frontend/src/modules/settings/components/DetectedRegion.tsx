import { useQuery } from "@tanstack/solid-query";
import { Match, Switch, type VoidComponent } from "solid-js";
import type { GameId } from "@/modules/games/games.types";
import { detectedRegionQueryOptions } from "@/modules/settings/settings.query";
import * as m from "@/paraglide/messages";

export interface DetectedRegionProps {
  gameId: GameId;
  uid: string;
}

/** Displays the server region derived from the game's UID. */
export const DetectedRegion: VoidComponent<DetectedRegionProps> = (props) => {
  const region = useQuery(() => detectedRegionQueryOptions(props.gameId, props.uid));

  return (
    <Switch>
      <Match when={region.data}>
        {(detected) => (
          <p class="text-xs text-zinc-500 dark:text-zinc-400">
            {m.settings_game_region_detected({ region: detected() })}
          </p>
        )}
      </Match>
      <Match when={region.isError}>
        <p class="text-xs text-amber-600 dark:text-amber-400">{m.settings_game_region_unknown()}</p>
      </Match>
    </Switch>
  );
};
