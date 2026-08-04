import { GameId } from "@/modules/games/games.types";
import type { AppConfig, GamesConfig } from "@/modules/settings/settings.types";

const GAME_CONFIG_KEYS: [GameId, keyof GamesConfig][] = [
  [GameId.GenshinImpact, "genshin_impact"],
  [GameId.HonkaiStarRail, "honkai_star_rail"],
  [GameId.ZenlessZoneZero, "zenless_zone_zero"],
  [GameId.WutheringWaves, "wuthering_waves"],
];

/** Returns the set of games enabled in config; empty while config is unloaded. */
export function enabledGamesFromConfig(config: AppConfig | undefined): Set<GameId> {
  return new Set<GameId>(
    GAME_CONFIG_KEYS.filter(([, key]) => config?.games[key]?.enabled).map(([id]) => id),
  );
}
