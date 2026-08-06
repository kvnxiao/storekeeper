import {
  GenshinResource,
  HsrResource,
  type ResourceType,
  WuwaResource,
  ZzzResource,
} from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import type { HoyolabConfigKey } from "@/modules/settings/settings.types";
import * as m from "@/paraglide/messages";

interface GameRegistryEntryBase {
  gameId: GameId;
  /** Paraglide message functions, called at render time to follow the locale. */
  name: () => string;
  description: () => string;
  resourceTypes: readonly ResourceType[];
}

/**
 * Discriminated on `provider` rather than flattened, so filtering by provider
 * keeps `configKey` narrow enough to build a TanStack Form field path from.
 */
export type GameRegistryEntry =
  | (GameRegistryEntryBase & {
      provider: "hoyolab";
      configKey: HoyolabConfigKey;
      supportsDailyRewards: true;
    })
  | (GameRegistryEntryBase & {
      provider: "kuro";
      configKey: "wuthering_waves";
      supportsDailyRewards: false;
    });

/**
 * The single description of every supported game, consumed by the dashboard,
 * the settings page, and the enabled-games derivation.
 *
 * `Record<GameId, …>` is the enforcement: a new `GameId` that is not registered
 * here fails to compile, and a renamed config key fails against `GamesConfig`.
 */
export const GAME_REGISTRY: Record<GameId, GameRegistryEntry> = {
  [GameId.GenshinImpact]: {
    gameId: GameId.GenshinImpact,
    provider: "hoyolab",
    configKey: "genshin_impact",
    supportsDailyRewards: true,
    name: m.game_genshin_name,
    description: m.settings_game_configure_genshin,
    resourceTypes: Object.values(GenshinResource),
  },
  [GameId.HonkaiStarRail]: {
    gameId: GameId.HonkaiStarRail,
    provider: "hoyolab",
    configKey: "honkai_star_rail",
    supportsDailyRewards: true,
    name: m.game_hsr_name,
    description: m.settings_game_configure_hsr,
    resourceTypes: Object.values(HsrResource),
  },
  [GameId.ZenlessZoneZero]: {
    gameId: GameId.ZenlessZoneZero,
    provider: "hoyolab",
    configKey: "zenless_zone_zero",
    supportsDailyRewards: true,
    name: m.game_zzz_name,
    description: m.settings_game_configure_zzz,
    resourceTypes: Object.values(ZzzResource),
  },
  [GameId.WutheringWaves]: {
    gameId: GameId.WutheringWaves,
    provider: "kuro",
    configKey: "wuthering_waves",
    supportsDailyRewards: false,
    name: m.game_wuwa_name,
    description: m.settings_game_configure_wuwa,
    resourceTypes: Object.values(WuwaResource),
  },
};

/** Every game in the order the dashboard and settings page list them. */
export const GAMES: readonly GameRegistryEntry[] = Object.values(GAME_REGISTRY);
