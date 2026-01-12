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
