import { listen } from "@tauri-apps/api/event";
import { createRoot } from "solid-js";
import { describe, expect, it, vi } from "vite-plus/test";
import { core, createCore } from "@/modules/core/core.state";

const { release } = vi.hoisted(() => ({ release: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(async () => "en") }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(async () => release) }));

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

  it("releases every backend listener when its root is disposed", async () => {
    const dispose = createRoot((disposeRoot) => {
      createCore().init();
      return disposeRoot;
    });
    const subscribed = vi.mocked(listen).mock.calls.length;
    expect(subscribed).toBeGreaterThan(0);

    dispose();

    await vi.waitFor(() => expect(release).toHaveBeenCalledTimes(subscribed));
  });
});
