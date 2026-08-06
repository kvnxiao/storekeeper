import { describe, expect, it, vi } from "vite-plus/test";
import { GameId } from "@/modules/games/games.types";
import type { AllResources } from "@/modules/resources/resources.types";
import {
  formatAbsoluteDateTime,
  formatTimeRemaining,
  getResourceLimitsForGame,
  isPastDateTime,
} from "@/modules/resources/resources.utils";

vi.mock("@/paraglide/messages", () => ({
  time_remaining_full: () => "Full",
}));

/** Mirrors the style logic in core.state.ts durationFormatter. */
function makeDurationFmt(locale: string): Intl.DurationFormat {
  return new Intl.DurationFormat(locale, {
    style: locale.startsWith("en") ? "narrow" : "short",
  });
}

/** Returns ISO string `deltaMs` in the future from `now`. */
function futureIso(now: number, deltaMs: number): string {
  return new Date(now + deltaMs).toISOString();
}

const MS = { s: 1_000, m: 60_000, h: 3_600_000, d: 86_400_000 };

describe("formatTimeRemaining - early returns", () => {
  const now = Date.now();
  const fmt = makeDurationFmt("en");

  it.each([
    ["null", null],
    ["undefined", undefined],
    ["empty string", ""],
    ["invalid date string", "not-a-date"],
    ["past datetime", new Date(now - MS.m).toISOString()],
    ["exact now (diffMs = 0)", new Date(now).toISOString()],
    ["sub-second remaining", futureIso(now, 500)],
  ])("returns Full for %s", (_label, datetime) => {
    expect(formatTimeRemaining(datetime, now, fmt)).toBe("Full");
  });
});

const LOCALE_CONFIGS = [
  {
    locale: "en",
    expected: {
      "1s": "1s",
      "45s": "45s",
      "59s": "59s",
      "1m": "1m",
      "1m 1s": "1m 1s",
      "50m 59s": "50m 59s",
      "59m 59s": "59m 59s",
      "1h": "1h",
      "1h 1m": "1h 1m",
      "1h 0m 1s": "1h",
      "2h 30m 45s": "2h 30m",
      "23h 59m": "23h 59m",
      "1d": "1d",
      "1d 5h 30m": "1d 5h 30m",
      "1d 0h 30m": "1d 30m",
      "1d 0h 0m 45s": "1d",
      "7d 12h": "7d 12h",
    },
  },
  {
    locale: "ja",
    expected: {
      "1s": "1 秒",
      "45s": "45 秒",
      "59s": "59 秒",
      "1m": "1 分",
      "1m 1s": "1 分 1 秒",
      "50m 59s": "50 分 59 秒",
      "59m 59s": "59 分 59 秒",
      "1h": "1 時間",
      "1h 1m": "1 時間 1 分",
      "1h 0m 1s": "1 時間",
      "2h 30m 45s": "2 時間 30 分",
      "23h 59m": "23 時間 59 分",
      "1d": "1 日",
      "1d 5h 30m": "1 日 5 時間 30 分",
      "1d 0h 30m": "1 日 30 分",
      "1d 0h 0m 45s": "1 日",
      "7d 12h": "7 日 12 時間",
    },
  },
  {
    locale: "ko",
    expected: {
      "1s": "1초",
      "45s": "45초",
      "59s": "59초",
      "1m": "1분",
      "1m 1s": "1분 1초",
      "50m 59s": "50분 59초",
      "59m 59s": "59분 59초",
      "1h": "1시간",
      "1h 1m": "1시간 1분",
      "1h 0m 1s": "1시간",
      "2h 30m 45s": "2시간 30분",
      "23h 59m": "23시간 59분",
      "1d": "1일",
      "1d 5h 30m": "1일 5시간 30분",
      "1d 0h 30m": "1일 30분",
      "1d 0h 0m 45s": "1일",
      "7d 12h": "7일 12시간",
    },
  },
  {
    locale: "zh-CN",
    expected: {
      "1s": "1秒",
      "45s": "45秒",
      "59s": "59秒",
      "1m": "1分钟",
      "1m 1s": "1分钟1秒",
      "50m 59s": "50分钟59秒",
      "59m 59s": "59分钟59秒",
      "1h": "1小时",
      "1h 1m": "1小时1分钟",
      "1h 0m 1s": "1小时",
      "2h 30m 45s": "2小时30分钟",
      "23h 59m": "23小时59分钟",
      "1d": "1天",
      "1d 5h 30m": "1天5小时30分钟",
      "1d 0h 30m": "1天30分钟",
      "1d 0h 0m 45s": "1天",
      "7d 12h": "7天12小时",
    },
  },
] as const;

/** Maps each test-case label to its deltaMs. */
const DURATION_DELTAS: Record<string, number> = {
  "1s": MS.s,
  "45s": 45 * MS.s,
  "59s": 59 * MS.s,
  "1m": MS.m,
  "1m 1s": MS.m + MS.s,
  "50m 59s": 50 * MS.m + 59 * MS.s,
  "59m 59s": 59 * MS.m + 59 * MS.s,
  "1h": MS.h,
  "1h 1m": MS.h + MS.m,
  "1h 0m 1s": MS.h + MS.s,
  "2h 30m 45s": 2 * MS.h + 30 * MS.m + 45 * MS.s,
  "23h 59m": 23 * MS.h + 59 * MS.m,
  "1d": MS.d,
  "1d 5h 30m": MS.d + 5 * MS.h + 30 * MS.m,
  "1d 0h 30m": MS.d + 30 * MS.m,
  "1d 0h 0m 45s": MS.d + 45 * MS.s,
  "7d 12h": 7 * MS.d + 12 * MS.h,
};

describe.each(LOCALE_CONFIGS)("formatTimeRemaining - $locale", ({ locale, expected }) => {
  const now = Date.now();
  const fmt = makeDurationFmt(locale);

  it.each(Object.entries(expected))("%s → %s", (label, output) => {
    const dt = futureIso(now, DURATION_DELTAS[label]);
    expect(formatTimeRemaining(dt, now, fmt)).toBe(output);
  });
});

describe("isPastDateTime", () => {
  const now = Date.now();

  it.each([
    ["null", null],
    ["undefined", undefined],
    ["empty string", ""],
    ["invalid date string", "not-a-date"],
  ])("counts %s as past, matching what the countdown renders", (_label, datetime) => {
    expect(isPastDateTime(datetime, now)).toBe(true);
  });

  it("counts the exact current instant as past", () => {
    expect(isPastDateTime(new Date(now).toISOString(), now)).toBe(true);
  });

  it("is false while the datetime is still ahead", () => {
    expect(isPastDateTime(futureIso(now, MS.m), now)).toBe(false);
  });
});

// Resource types are spelled out rather than imported from games.constants,
// which builds its display-name table from the message module this file mocks.
describe("getResourceLimitsForGame", () => {
  const resources: AllResources = {
    games: {
      [GameId.GenshinImpact]: [
        {
          type: "resin",
          data: { current: 100, max: 200, fullAt: "", regenRateSeconds: 480 },
        },
        {
          type: "parametric_transformer",
          data: { isReady: false, readyAt: "" },
        },
      ],
    },
  };

  it("reports max and regen rate for stamina resources", () => {
    expect(getResourceLimitsForGame(resources, GameId.GenshinImpact)).toEqual({
      resin: { maxValue: 200, regenRateSeconds: 480 },
    });
  });

  it("skips resources that carry no value ceiling", () => {
    const limits = getResourceLimitsForGame(resources, GameId.GenshinImpact);
    expect(limits.parametric_transformer).toBeUndefined();
  });

  it("is empty for a game with no snapshot yet", () => {
    expect(getResourceLimitsForGame(undefined, GameId.GenshinImpact)).toEqual({});
    expect(getResourceLimitsForGame(resources, GameId.HonkaiStarRail)).toEqual({});
  });
});

describe("formatAbsoluteDateTime", () => {
  const now = Date.now();
  const timeOnlyFmt = new Intl.DateTimeFormat("en", { hour: "numeric", minute: "2-digit" });
  const weekdayTimeFmt = new Intl.DateTimeFormat("en", {
    weekday: "short",
    hour: "numeric",
    minute: "2-digit",
  });

  it.each([
    ["null", null],
    ["undefined", undefined],
    ["empty string", ""],
    ["invalid date string", "not-a-date"],
  ])("returns null for %s instead of throwing", (_label, datetime) => {
    expect(formatAbsoluteDateTime(datetime, now, timeOnlyFmt, weekdayTimeFmt)).toBeNull();
  });

  it("formats same-day targets as time only", () => {
    const target = new Date(now);
    expect(formatAbsoluteDateTime(target.toISOString(), now, timeOnlyFmt, weekdayTimeFmt)).toBe(
      timeOnlyFmt.format(target),
    );
  });

  it("formats other-day targets with the weekday", () => {
    const target = new Date(now + 3 * MS.d);
    expect(formatAbsoluteDateTime(target.toISOString(), now, timeOnlyFmt, weekdayTimeFmt)).toBe(
      weekdayTimeFmt.format(target),
    );
  });
});
