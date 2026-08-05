import { ZzzResource } from "@/modules/games/games.constants";
import { createStaminaResource } from "@/modules/games/games.primitives";
import { GameId } from "@/modules/games/games.types";

/** Reactive Zenless Zone Zero resource selectors over the resources query. */
export function createZzzResources() {
  const [battery, batteryTime] = createStaminaResource(GameId.ZenlessZoneZero, ZzzResource.Battery);

  return { battery, batteryTime };
}
