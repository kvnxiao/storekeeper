import type {
  GenshinResourceType,
  HsrResourceType,
  WuwaResourceType,
  ZzzResourceType,
} from "@/modules/games/games.constants";

/** Game ID const object matching Rust GameId */
export const GameId = {
  GenshinImpact: "GENSHIN_IMPACT",
  HonkaiStarRail: "HONKAI_STAR_RAIL",
  ZenlessZoneZero: "ZENLESS_ZONE_ZERO",
  WutheringWaves: "WUTHERING_WAVES",
} as const;

export type GameId = (typeof GameId)[keyof typeof GameId];

/** Maps each GameId to its valid resource type strings */
export interface GameResourceTypeMap {
  GENSHIN_IMPACT: GenshinResourceType;
  HONKAI_STAR_RAIL: HsrResourceType;
  ZENLESS_ZONE_ZERO: ZzzResourceType;
  WUTHERING_WAVES: WuwaResourceType;
}
