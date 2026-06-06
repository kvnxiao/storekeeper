import "@formatjs/intl-durationformat/polyfill.js";
import { describe, expect, it, vi } from "vite-plus/test";
import { formatTimeRemaining } from "./resources.utils";

vi.mock("@/paraglide/messages", () => ({
  time_remaining_full: () => "Full",
}));

/** Mirrors the style logic in core.atoms.ts durationFormatter. */
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

// ---------------------------------------------------------------------------
// "Full" early-return cases — locale-independent
// ---------------------------------------------------------------------------

describe("formatTimeRemaining — early returns", () => {
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

// ---------------------------------------------------------------------------
// Duration formatting — parametrized per locale
// ---------------------------------------------------------------------------

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
  "7d 12h": 7 * MS.d + 12 * MS.h,
};

describe.each(LOCALE_CONFIGS)("formatTimeRemaining — $locale", ({ locale, expected }) => {
  const now = Date.now();
  const fmt = makeDurationFmt(locale);

  it.each(Object.entries(expected))("%s → %s", (label, output) => {
    const dt = futureIso(now, DURATION_DELTAS[label]);
    expect(formatTimeRemaining(dt, now, fmt)).toBe(output);
  });
});
