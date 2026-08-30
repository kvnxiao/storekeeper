import { describe, expect, it } from "vite-plus/test";
import { GAME_REGISTRY } from "@/modules/games/games.registry";
import { GameId } from "@/modules/games/games.types";
import type { AppConfig, ResourceNotificationConfig } from "@/modules/settings/settings.types";
import {
  DEFAULT_NOTIFICATION_CONFIG,
  emptyHoyolabConfig,
  emptyWuwaConfig,
  enabledGamesFromConfig,
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

const NO_GAMES: AppConfig["games"] = {
  genshin_impact: null,
  honkai_star_rail: null,
  zenless_zone_zero: null,
  wuthering_waves: null,
};

function config(games: Partial<AppConfig["games"]>): AppConfig {
  return {
    general: {
      poll_interval_secs: 600,
      start_minimized: false,
      log_level: "info",
      language: null,
      autostart: false,
    },
    games: { ...NO_GAMES, ...games },
  };
}

describe("enabledGamesFromConfig", () => {
  it("is empty while the config has not loaded", () => {
    expect(enabledGamesFromConfig(undefined).size).toBe(0);
  });

  it("returns only the games whose config is enabled", () => {
    const enabled = enabledGamesFromConfig(
      config({
        genshin_impact: { ...emptyHoyolabConfig([]), enabled: true, uid: "1" },
        honkai_star_rail: { ...emptyHoyolabConfig([]), enabled: false, uid: "2" },
        wuthering_waves: { ...emptyWuwaConfig([]), enabled: true, uid: "3" },
      }),
    );

    expect([...enabled]).toEqual([GameId.GenshinImpact, GameId.WutheringWaves]);
  });

  it("treats an unconfigured game as disabled", () => {
    expect(enabledGamesFromConfig(config({})).size).toBe(0);
  });
});

describe("emptyHoyolabConfig", () => {
  it("tracks the game's full resource set, matching the backend default", () => {
    const genshin = GAME_REGISTRY[GameId.GenshinImpact].resourceTypes;

    expect(emptyHoyolabConfig(genshin).tracked_resources).toEqual([
      "resin",
      "parametric_transformer",
      "realm_currency",
      "expeditions",
    ]);
  });

  it("copies the resource types instead of aliasing the registry", () => {
    const genshin = GAME_REGISTRY[GameId.GenshinImpact].resourceTypes;
    const config = emptyHoyolabConfig(genshin);

    config.tracked_resources.pop();

    expect(genshin).toHaveLength(4);
  });

  it("leaves every nullable field null so the payload matches the wire shape", () => {
    expect(emptyHoyolabConfig([])).toEqual({
      enabled: false,
      uid: "",
      tracked_resources: [],
      auto_claim_daily_rewards: false,
      auto_claim_time: null,
      notifications: {},
    });
  });
});

describe("emptyWuwaConfig", () => {
  it("tracks the game's full resource set and omits the daily-claim fields", () => {
    const wuwa = GAME_REGISTRY[GameId.WutheringWaves].resourceTypes;

    expect(emptyWuwaConfig(wuwa)).toEqual({
      enabled: false,
      uid: "",
      tracked_resources: ["waveplates"],
      notifications: {},
    });
  });
});

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
