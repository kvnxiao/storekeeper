import { describe, expect, it } from "vite-plus/test";
import { extractClaimStatus, utc8Date } from "@/modules/daily-rewards/daily-rewards.utils";
import { GameId } from "@/modules/games/games.types";

describe("extractClaimStatus", () => {
  it("maps each game's signed flag", () => {
    const status = extractClaimStatus({
      games: {
        [GameId.GenshinImpact]: { info: { is_signed: true } },
        [GameId.HonkaiStarRail]: { info: { is_signed: false } },
      },
    });
    expect(status.get(GameId.GenshinImpact)).toBe(true);
    expect(status.get(GameId.HonkaiStarRail)).toBe(false);
  });

  it("omits games whose status is unknown rather than assuming unclaimed", () => {
    const status = extractClaimStatus({
      games: {
        [GameId.GenshinImpact]: {},
        [GameId.HonkaiStarRail]: { info: {} },
      },
    });
    expect(status.has(GameId.GenshinImpact)).toBe(false);
    expect(status.has(GameId.HonkaiStarRail)).toBe(false);
  });

  it("returns an empty map when the payload carries no games", () => {
    expect(extractClaimStatus({}).size).toBe(0);
  });
});

describe("utc8Date", () => {
  it("is still the previous date just before midnight UTC+8", () => {
    expect(utc8Date(Temporal.Instant.from("2026-08-05T15:59:00Z")).toString()).toBe("2026-08-05");
  });

  it("rolls over at midnight UTC+8, which is 16:00 UTC", () => {
    expect(utc8Date(Temporal.Instant.from("2026-08-05T16:00:00Z")).toString()).toBe("2026-08-06");
  });
});
