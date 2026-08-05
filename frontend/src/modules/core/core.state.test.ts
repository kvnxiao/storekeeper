import { describe, expect, it } from "vitest";
import { core } from "@/modules/core/core.state";

describe("core state", () => {
  it("holds views back until the backend locale has been resolved", () => {
    expect(core.localeReady()).toBe(false);
  });

  it("moves the Intl formatters onto an applied locale", async () => {
    await core.setAppLocale("ja");

    expect(core.locale()).toBe("ja");
    expect(core.durationFormatter().resolvedOptions().locale).toMatch(/^ja/);
    expect(core.timeOnlyFormatter().resolvedOptions().locale).toMatch(/^ja/);
    expect(core.weekdayTimeFormatter().resolvedOptions().locale).toMatch(/^ja/);
  });

  it("ignores a locale the app has no messages for", async () => {
    await core.setAppLocale("ja");
    await core.setAppLocale("de-CH");

    expect(core.locale()).toBe("ja");
  });
});
