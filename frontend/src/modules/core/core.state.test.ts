import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createRoot } from "solid-js";
import { describe, expect, it, vi } from "vite-plus/test";
import { core, createCore } from "@/modules/core/core.state";

const { release, dailyRewardsInit } = vi.hoisted(() => ({
  release: vi.fn(),
  dailyRewardsInit: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(async () => "en") }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(async () => release) }));
vi.mock("@/modules/daily-rewards/daily-rewards.state", () => ({
  dailyRewardsState: { init: dailyRewardsInit },
}));

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
    expect(core.timeWithSecondsFormatter().resolvedOptions().locale).toMatch(/^ja/);
  });

  it("ignores a locale the app has no messages for", async () => {
    await core.setAppLocale("ja");
    await core.setAppLocale("de-CH");

    expect(core.locale()).toBe("ja");
  });

  it("resolves the locale from the shell without touching the dashboard", async () => {
    vi.mocked(listen).mockClear();
    vi.mocked(invoke).mockClear();
    dailyRewardsInit.mockClear();

    const { instance, dispose } = createRoot((disposeRoot) => ({
      instance: createCore(),
      dispose: disposeRoot,
    }));
    instance.initShell();

    await vi.waitFor(() => expect(instance.localeReady()).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_effective_locale");
    expect(listen).not.toHaveBeenCalled();
    expect(dailyRewardsInit).not.toHaveBeenCalled();

    dispose();
  });

  it("registers the backend listeners once across repeated dashboard calls", () => {
    vi.mocked(listen).mockClear();
    dailyRewardsInit.mockClear();

    const { instance, dispose } = createRoot((disposeRoot) => ({
      instance: createCore(),
      dispose: disposeRoot,
    }));
    instance.initDashboard();
    const subscribed = vi.mocked(listen).mock.calls.length;
    instance.initDashboard();

    expect(subscribed).toBeGreaterThan(0);
    expect(vi.mocked(listen).mock.calls).toHaveLength(subscribed);
    expect(dailyRewardsInit).toHaveBeenCalledTimes(1);

    dispose();
  });

  it("releases every backend listener when its root is disposed", async () => {
    vi.mocked(listen).mockClear();
    release.mockClear();

    const dispose = createRoot((disposeRoot) => {
      createCore().initDashboard();
      return disposeRoot;
    });
    const subscribed = vi.mocked(listen).mock.calls.length;
    expect(subscribed).toBeGreaterThan(0);

    dispose();

    await vi.waitFor(() => expect(release).toHaveBeenCalledTimes(subscribed));
  });
});
