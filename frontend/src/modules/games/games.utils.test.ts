import { describe, expect, it } from "vitest";
import { GenshinResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import {
  fillStaminaWhenDue,
  readyCooldownWhenDue,
  selectResource,
} from "@/modules/games/games.utils";
import {
  type AllResources,
  type CooldownResource,
  isCooldownResource,
  isStaminaResource,
  type StaminaResource,
} from "@/modules/resources/resources.types";

const NOW = Date.UTC(2026, 7, 5, 12, 0, 0);
const PAST = new Date(NOW - 60_000).toISOString();
const FUTURE = new Date(NOW + 60_000).toISOString();

function stamina(overrides: Partial<StaminaResource> = {}): StaminaResource {
  return { current: 100, max: 160, fullAt: FUTURE, regenRateSeconds: 480, ...overrides };
}

function cooldown(overrides: Partial<CooldownResource> = {}): CooldownResource {
  return { isReady: false, readyAt: FUTURE, ...overrides };
}

function resources(type: string, data: StaminaResource | CooldownResource): AllResources {
  return { games: { [GameId.GenshinImpact]: [{ type, data }] } };
}

describe("selectResource", () => {
  it("returns the matching resource data", () => {
    const data = stamina();
    const selected = selectResource(
      resources(GenshinResource.Resin, data),
      GameId.GenshinImpact,
      GenshinResource.Resin,
      isStaminaResource,
    );
    expect(selected).toEqual(data);
  });

  it("returns null when the game has no resources", () => {
    expect(
      selectResource({}, GameId.GenshinImpact, GenshinResource.Resin, isStaminaResource),
    ).toBeNull();
  });

  it("returns null when no resource has the requested type", () => {
    expect(
      selectResource(
        resources(GenshinResource.RealmCurrency, stamina()),
        GameId.GenshinImpact,
        GenshinResource.Resin,
        isStaminaResource,
      ),
    ).toBeNull();
  });

  it("returns null when the guard rejects the payload", () => {
    expect(
      selectResource(
        resources(GenshinResource.Resin, stamina()),
        GameId.GenshinImpact,
        GenshinResource.Resin,
        isCooldownResource,
      ),
    ).toBeNull();
  });
});

describe("fillStaminaWhenDue", () => {
  it("fills to max once fullAt has passed", () => {
    expect(fillStaminaWhenDue(stamina({ fullAt: PAST }), NOW)?.current).toBe(160);
  });

  it("leaves the backend value alone while fullAt is still ahead", () => {
    expect(fillStaminaWhenDue(stamina(), NOW)?.current).toBe(100);
  });

  it("returns the same object when already full", () => {
    const full = stamina({ current: 160, fullAt: PAST });
    expect(fillStaminaWhenDue(full, NOW)).toBe(full);
  });

  it("treats a missing resource as nothing to fill", () => {
    expect(fillStaminaWhenDue(null, NOW)).toBeNull();
  });

  it("fills when fullAt is unparseable, matching the countdown's fallback", () => {
    expect(fillStaminaWhenDue(stamina({ fullAt: "not a date" }), NOW)?.current).toBe(160);
  });
});

describe("readyCooldownWhenDue", () => {
  it("marks ready once readyAt has passed", () => {
    expect(readyCooldownWhenDue(cooldown({ readyAt: PAST }), NOW)?.isReady).toBe(true);
  });

  it("stays pending while readyAt is still ahead", () => {
    expect(readyCooldownWhenDue(cooldown(), NOW)?.isReady).toBe(false);
  });

  it("returns the same object when already ready", () => {
    const ready = cooldown({ isReady: true });
    expect(readyCooldownWhenDue(ready, NOW)).toBe(ready);
  });

  it("treats a missing resource as nothing to mark", () => {
    expect(readyCooldownWhenDue(null, NOW)).toBeNull();
  });
});
