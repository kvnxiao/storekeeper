import { WuwaResource } from "@/modules/games/games.constants";
import { createStaminaResource } from "@/modules/games/games.primitives";
import { GameId } from "@/modules/games/games.types";

/** Reactive Wuthering Waves resource selectors over the resources query. */
export function createWuwaResources() {
  const [waveplates, waveplatesTime] = createStaminaResource(
    GameId.WutheringWaves,
    WuwaResource.Waveplates,
  );

  return { waveplates, waveplatesTime };
}
