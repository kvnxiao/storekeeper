import { describe, expect, it } from "vite-plus/test";
import { LOG_LEVELS } from "@/modules/logs/logs.constants";
import {
  admitsLevel,
  containsSelection,
  errorText,
  fieldSummary,
  filterEntries,
  isAtBottom,
  type LogEntry,
  logLevelOptions,
  parseLogLine,
  parseLogLines,
  parseLogTimestamp,
} from "@/modules/logs/logs.utils";

const WELL_FORMED = JSON.stringify({
  timestamp: "2026-08-21T04:15:09.123456Z",
  level: "INFO",
  fields: { message: "Auto-claim successful", game_id: "GenshinImpact" },
  target: "storekeeper_app_tauri::scheduled_claim",
});

function entry(overrides: Partial<LogEntry> = {}): LogEntry {
  return {
    timestamp: "2026-08-21T04:15:09.123456Z",
    level: "info",
    target: "storekeeper_app_tauri::polling",
    message: "Refreshed resources",
    fields: {},
    ...overrides,
  };
}

describe("parseLogLine", () => {
  it("splits a well-formed line into the typed entry", () => {
    expect(parseLogLine(WELL_FORMED)).toEqual({
      timestamp: "2026-08-21T04:15:09.123456Z",
      level: "info",
      target: "storekeeper_app_tauri::scheduled_claim",
      message: "Auto-claim successful",
      fields: { game_id: "GenshinImpact" },
    });
  });

  it("returns null for a line truncated mid-write", () => {
    expect(parseLogLine('{"timestamp":"2026-08-21T04:15:09.1234')).toBeNull();
  });

  it("returns null for a line that is not JSON", () => {
    expect(parseLogLine("2026-08-21T04:15:09Z  INFO storekeeper: plain text")).toBeNull();
  });

  it("returns null for JSON carrying no recognizable level", () => {
    expect(parseLogLine(JSON.stringify({ level: "chatty", fields: {} }))).toBeNull();
  });

  it("substitutes empty strings for absent optional members", () => {
    expect(parseLogLine(JSON.stringify({ level: "ERROR" }))).toEqual({
      timestamp: "",
      level: "error",
      target: "",
      message: "",
      fields: {},
    });
  });
});

describe("parseLogLines", () => {
  it("drops the unparseable lines and keeps the rest", () => {
    expect(parseLogLines([WELL_FORMED, "not json", WELL_FORMED])).toHaveLength(2);
  });
});

describe("admitsLevel", () => {
  it("admits a level at least as severe as the minimum", () => {
    expect(admitsLevel("info", "error")).toBe(true);
    expect(admitsLevel("info", "warn")).toBe(true);
    expect(admitsLevel("info", "info")).toBe(true);
  });

  it("rejects a level less severe than the minimum", () => {
    expect(admitsLevel("info", "debug")).toBe(false);
    expect(admitsLevel("info", "trace")).toBe(false);
  });

  it("admits every level at trace", () => {
    expect(admitsLevel("trace", "trace")).toBe(true);
    expect(admitsLevel("trace", "error")).toBe(true);
  });

  it("admits only errors at error", () => {
    expect(admitsLevel("error", "error")).toBe(true);
    expect(admitsLevel("error", "warn")).toBe(false);
  });
});

describe("filterEntries", () => {
  const entries = [
    entry({ level: "debug", message: "Fetching daily reward info" }),
    entry({ level: "error", message: "Auto-claim failed", target: "scheduled_claim" }),
    entry({ level: "info", message: "Refreshed resources", fields: { risk_code: 5001 } }),
  ];

  it("keeps only the entries at or above the minimum level", () => {
    expect(filterEntries(entries, "info", "").map((e) => e.level)).toEqual(["error", "info"]);
  });

  it("matches the search against the message", () => {
    expect(filterEntries(entries, "trace", "auto-claim")).toHaveLength(1);
  });

  it("matches the search against the target", () => {
    expect(filterEntries(entries, "trace", "scheduled_claim")).toHaveLength(1);
  });

  it("matches the search against a structured field", () => {
    expect(filterEntries(entries, "trace", "5001")).toHaveLength(1);
  });

  it("ignores surrounding whitespace in the search", () => {
    expect(filterEntries(entries, "trace", "   ")).toHaveLength(3);
  });
});

describe("parseLogTimestamp", () => {
  it("reads the backend's UTC stamp as the instant it names", () => {
    expect(parseLogTimestamp("2026-08-21T04:15:09.123456Z")?.toISOString()).toBe(
      "2026-08-21T04:15:09.123Z",
    );
  });

  it("returns null for a value that is not a date", () => {
    expect(parseLogTimestamp("unknown")).toBeNull();
  });

  it("returns null for an absent timestamp", () => {
    expect(parseLogTimestamp("")).toBeNull();
  });
});

describe("fieldSummary", () => {
  it("is empty for an entry with no structured fields", () => {
    expect(fieldSummary(entry())).toBe("");
  });

  it("renders a string field without quoting it", () => {
    expect(fieldSummary(entry({ fields: { game_id: "GenshinImpact" } }))).toBe(
      "game_id=GenshinImpact",
    );
  });

  it("JSON-encodes a field that is not a string", () => {
    expect(fieldSummary(entry({ fields: { risk_code: 5001, retry: false } }))).toBe(
      "risk_code=5001 retry=false",
    );
  });
});

describe("errorText", () => {
  it("reads the message off a rejected command", () => {
    expect(errorText({ code: "IO_ERROR", message: "failed to open the log file" })).toBe(
      "failed to open the log file",
    );
  });

  it("falls back to the value itself when it carries no message", () => {
    expect(errorText("boom")).toBe("boom");
    expect(errorText(null)).toBe("null");
  });
});

describe("logLevelOptions", () => {
  it("offers every level, most to least severe", () => {
    expect(logLevelOptions().map((option) => option.id)).toEqual([...LOG_LEVELS]);
  });

  it("labels every option", () => {
    expect(logLevelOptions().every((option) => option.label.length > 0)).toBe(true);
  });
});

describe("isAtBottom", () => {
  it("treats a container scrolled to its bottom edge as at the bottom", () => {
    expect(isAtBottom({ scrollHeight: 1000, scrollTop: 700, clientHeight: 300 })).toBe(true);
  });

  it("allows a small gap so a fractional scroll position still counts", () => {
    expect(isAtBottom({ scrollHeight: 1000, scrollTop: 690, clientHeight: 300 })).toBe(true);
  });

  it("reports a container scrolled away from the bottom", () => {
    expect(isAtBottom({ scrollHeight: 1000, scrollTop: 200, clientHeight: 300 })).toBe(false);
  });

  it("treats a container shorter than its viewport as at the bottom", () => {
    expect(isAtBottom({ scrollHeight: 120, scrollTop: 0, clientHeight: 300 })).toBe(true);
  });
});

describe("containsSelection", () => {
  it("reports no selection when nothing is selected", () => {
    document.getSelection()?.removeAllRanges();
    expect(containsSelection(document.body)).toBe(false);
  });

  it("reports a selection anchored inside the element", () => {
    const host = document.createElement("div");
    host.textContent = "claim registered";
    document.body.append(host);

    const range = document.createRange();
    range.selectNodeContents(host);
    const selection = document.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(range);

    expect(containsSelection(host)).toBe(true);

    selection?.removeAllRanges();
    host.remove();
  });

  it("ignores a selection anchored outside the element", () => {
    const selected = document.createElement("div");
    selected.textContent = "claim registered";
    const other = document.createElement("div");
    document.body.append(selected, other);

    const range = document.createRange();
    range.selectNodeContents(selected);
    const selection = document.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(range);

    expect(containsSelection(other)).toBe(false);

    selection?.removeAllRanges();
    selected.remove();
    other.remove();
  });
});
