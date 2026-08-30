import { describe, expect, it } from "vite-plus/test";
import { GenshinResource, type ResourceType } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import {
  estimateStaminaCurrent,
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

const NOW = Temporal.Instant.from("2026-08-05T12:00:00Z");
const PAST = NOW.subtract({ minutes: 1 }).toString();
const FUTURE = NOW.add({ minutes: 1 }).toString();

function stamina(overrides: Partial<StaminaResource> = {}): StaminaResource {
  return {
    current: 100,
    max: 160,
    fullAt: FUTURE,
    regenRateSeconds: 480,
    regenStepUnits: 1,
    ...overrides,
  };
}

function cooldown(overrides: Partial<CooldownResource> = {}): CooldownResource {
  return { isReady: false, readyAt: FUTURE, ...overrides };
}

function resources(type: ResourceType, data: StaminaResource | CooldownResource): AllResources {
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

describe("estimateStaminaCurrent", () => {
  const realm = (secondsToFull: number): StaminaResource =>
    stamina({
      current: 1980,
      max: 2400,
      fullAt: NOW.add({ seconds: secondsToFull }).toString(),
      regenRateSeconds: 3600,
      regenStepUnits: 30,
    });

  it("fills to max once fullAt has passed", () => {
    expect(estimateStaminaCurrent(stamina({ fullAt: PAST }), NOW)?.current).toBe(160);
  });

  it("counts a part-way unit as still owed", () => {
    const oneUnitLeft = stamina({ fullAt: NOW.add({ minutes: 3 }).toString() });
    expect(estimateStaminaCurrent(oneUnitLeft, NOW)?.current).toBe(159);
  });

  it("advances past the backend value as units tick", () => {
    const twoUnitsLeft = stamina({ fullAt: NOW.add({ minutes: 9 }).toString() });
    expect(estimateStaminaCurrent(twoUnitsLeft, NOW)?.current).toBe(158);
  });

  it("holds flat across a multi-unit step", () => {
    expect(estimateStaminaCurrent(realm(14 * 3600), NOW)?.current).toBe(1980);
    expect(estimateStaminaCurrent(realm(13 * 3600 + 1), NOW)?.current).toBe(1980);
  });

  it("jumps a whole step at the step boundary", () => {
    expect(estimateStaminaCurrent(realm(13 * 3600), NOW)?.current).toBe(2010);
  });

  it("returns the same object when already full", () => {
    const full = stamina({ current: 160, fullAt: PAST });
    expect(estimateStaminaCurrent(full, NOW)).toBe(full);
  });

  it("treats a missing resource as nothing to estimate", () => {
    expect(estimateStaminaCurrent(null, NOW)).toBeNull();
  });

  it("fills when fullAt is unparseable, matching the countdown's fallback", () => {
    expect(estimateStaminaCurrent(stamina({ fullAt: "not a date" }), NOW)?.current).toBe(160);
  });

  it("keeps the backend value when the step interval is zero", () => {
    const noRate = stamina({ regenRateSeconds: 0 });
    expect(estimateStaminaCurrent(noRate, NOW)).toBe(noRate);
  });

  it("never reads below the polled value when a step is clipped short at max", () => {
    const clipped = realm(47 * 3600);
    expect(estimateStaminaCurrent({ ...clipped, current: 1000 }, NOW)?.current).toBe(1000);
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
