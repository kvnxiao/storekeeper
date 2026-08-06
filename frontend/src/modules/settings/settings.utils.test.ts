import { describe, expect, it } from "vite-plus/test";
import type { ResourceNotificationConfig } from "@/modules/settings/settings.types";
import {
  DEFAULT_NOTIFICATION_CONFIG,
  enabledNotificationConfig,
  getNotifyMode,
  withNotifyAtValue,
  withNotifyMinutes,
  withNotifyMode,
} from "@/modules/settings/settings.utils";

const base: ResourceNotificationConfig = {
  enabled: false,
  notify_minutes_before_full: 30,
  notify_at_value: null,
  cooldown_minutes: 15,
};

describe("getNotifyMode", () => {
  it("is value when notify_at_value is set", () => {
    expect(getNotifyMode({ ...base, notify_at_value: 100 })).toBe("value");
  });

  it("is minutes when notify_at_value is null or absent", () => {
    expect(getNotifyMode(base)).toBe("minutes");
    expect(getNotifyMode({ enabled: true, cooldown_minutes: 0 })).toBe("minutes");
  });
});

describe("enabledNotificationConfig", () => {
  it("returns the default config when none exists yet", () => {
    expect(enabledNotificationConfig(undefined, true)).toEqual(DEFAULT_NOTIFICATION_CONFIG);
  });

  it("keeps thresholds for stamina resources", () => {
    expect(enabledNotificationConfig(base, true)).toEqual({ ...base, enabled: true });
  });

  it("clears both thresholds for cooldown resources", () => {
    expect(enabledNotificationConfig({ ...base, notify_at_value: 100 }, false)).toEqual({
      ...base,
      enabled: true,
      notify_minutes_before_full: null,
      notify_at_value: null,
    });
  });
});

describe("withNotifyMode", () => {
  it("switching to value clears minutes and defaults the value to 1", () => {
    expect(withNotifyMode(base, "value")).toEqual({
      ...base,
      notify_minutes_before_full: null,
      notify_at_value: 1,
    });
  });

  it("switching to minutes clears value and defaults minutes to 0 when unset", () => {
    const valueMode = { ...base, notify_minutes_before_full: null, notify_at_value: 100 };
    expect(withNotifyMode(valueMode, "minutes")).toEqual({
      ...base,
      notify_at_value: null,
      notify_minutes_before_full: 0,
    });
  });
});

describe("threshold setters keep the fields mutually exclusive", () => {
  it("withNotifyAtValue clears minutes", () => {
    expect(withNotifyAtValue(base, 120)).toEqual({
      ...base,
      notify_at_value: 120,
      notify_minutes_before_full: null,
    });
  });

  it("withNotifyMinutes clears value", () => {
    const valueMode = { ...base, notify_minutes_before_full: null, notify_at_value: 100 };
    expect(withNotifyMinutes(valueMode, 45)).toEqual({
      ...valueMode,
      notify_minutes_before_full: 45,
      notify_at_value: null,
    });
  });
});
