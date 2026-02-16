import type { GameId, GameMetadata } from "@/modules/games/games.types";

/** Game metadata lookup */
export const GAME_METADATA: Record<GameId, GameMetadata> = {
  GENSHIN_IMPACT: { title: "Genshin Impact", shortId: "genshin" },
  HONKAI_STAR_RAIL: { title: "Honkai: Star Rail", shortId: "hsr" },
  ZENLESS_ZONE_ZERO: { title: "Zenless Zone Zero", shortId: "zzz" },
  WUTHERING_WAVES: { title: "Wuthering Waves", shortId: "wuwa" },
};

/** Ordered list of games */
export const GAME_ORDER: GameId[] = [
  "GENSHIN_IMPACT",
  "HONKAI_STAR_RAIL",
  "ZENLESS_ZONE_ZERO",
  "WUTHERING_WAVES",
];

/** Resource types that are stamina-based (support value-threshold notifications) */
export const STAMINA_RESOURCE_TYPES: ReadonlySet<string> = new Set([
  "resin",
  "realm_currency",
  "trailblaze_power",
  "battery",
  "waveplates",
]);

/** Human-readable display names for resource type tags */
export const RESOURCE_DISPLAY_NAMES: Record<string, string> = {
  resin: "Original Resin",
  parametric_transformer: "Parametric Transformer",
  realm_currency: "Realm Currency",
  expeditions: "Expeditions",
  trailblaze_power: "Trailblaze Power",
  battery: "Battery",
  waveplates: "Waveplates",
};
