import { describe, expect, it } from "vite-plus/test";
import { parseInstant } from "@/modules/core/core.utils";

describe("parseInstant", () => {
  it("keeps the sub-millisecond digits the backend writes", () => {
    expect(parseInstant("2026-08-21T04:15:09.123456Z")?.toString()).toBe(
      "2026-08-21T04:15:09.123456Z",
    );
  });

  it("reads an offset timestamp as the instant it names", () => {
    expect(parseInstant("2026-08-05T20:00:00+08:00")?.toString()).toBe("2026-08-05T12:00:00Z");
  });

  it.each([
    ["null", null],
    ["undefined", undefined],
    ["empty string", ""],
    ["a value that is not a date", "unknown"],
    ["a datetime carrying no offset", "2026-08-21T04:15:09"],
  ])("returns null for %s", (_label, value) => {
    expect(parseInstant(value)).toBeNull();
  });
});
