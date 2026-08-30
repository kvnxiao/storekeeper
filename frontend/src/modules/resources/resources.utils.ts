import { parseInstant } from "@/modules/core/core.utils";
import type { ResourceType } from "@/modules/games/games.constants";
import type { GameId } from "@/modules/games/games.types";
import {
  type AllResources,
  isStaminaResource,
  type ResourceLimits,
} from "@/modules/resources/resources.types";
import * as m from "@/paraglide/messages";

/**
 * Formats the time left until an ISO 8601 `datetime` as a duration ("2h 13m"),
 * or "Full" once it has passed. `now` comes from the tick accessor so every
 * countdown on screen measures against the same instant.
 */
export function formatTimeRemaining(
  datetime: string | null | undefined,
  now: Temporal.Instant,
  durationFmt: Intl.DurationFormat,
): string {
  const target = parseInstant(datetime);
  if (target === null) {
    return m.time_remaining_full();
  }
  // Instant differences top out at hours; balancing to days afterwards keeps
  // every day exactly 24 hours.
  const remaining = now
    .until(target, { largestUnit: "hour", smallestUnit: "second", roundingMode: "trunc" })
    .round({ largestUnit: "day" });
  if (remaining.sign <= 0) {
    return m.time_remaining_full();
  }
  const { days, hours, minutes, seconds } = remaining;

  return durationFmt.format({
    days: days > 0 ? days : undefined,
    hours: days > 0 || hours > 0 ? hours : undefined,
    minutes: minutes > 0 ? minutes : undefined,
    seconds: days === 0 && hours === 0 && seconds > 0 ? seconds : undefined,
  });
}

/**
 * Checks whether an ISO 8601 `datetime` is in the past. Missing or unparseable
 * datetimes count as past, matching what the countdown renders for them.
 */
export function isPastDateTime(
  datetime: string | null | undefined,
  now: Temporal.Instant,
): boolean {
  const target = parseInstant(datetime);
  return target === null || Temporal.Instant.compare(target, now) <= 0;
}

/**
 * Formats an ISO 8601 `datetime` as a clock time ("3:17 PM"), prefixed with the
 * weekday when it does not fall on the same calendar day as `now`. Returns null
 * when there is nothing parseable to format.
 */
export function formatAbsoluteDateTime(
  datetime: string | null | undefined,
  now: Temporal.Instant,
  timeOnlyFmt: Intl.DateTimeFormat,
  weekdayTimeFmt: Intl.DateTimeFormat,
): string | null {
  const target = parseInstant(datetime);
  if (target === null) {
    return null;
  }
  const timeZone = Temporal.Now.timeZoneId();
  const isToday = target
    .toZonedDateTimeISO(timeZone)
    .toPlainDate()
    .equals(now.toZonedDateTimeISO(timeZone).toPlainDate());
  return isToday ? timeOnlyFmt.format(target) : weekdayTimeFmt.format(target);
}

/** Returns the fill duration at the accrual step, rounded down to whole minutes. */
export function minutesToFull(limits: ResourceLimits): number {
  const stepUnits = Math.max(limits.regenStepUnits, 1);
  return Math.floor((Math.ceil(limits.maxValue / stepUnits) * limits.regenRateSeconds) / 60);
}

/** Extracts per-resource stamina input constraints for a game. */
export function getResourceLimitsForGame(
  resources: AllResources | undefined,
  gameId: GameId,
): Partial<Record<ResourceType, ResourceLimits>> {
  const limits: Partial<Record<ResourceType, ResourceLimits>> = {};
  for (const resource of resources?.games?.[gameId] ?? []) {
    if (isStaminaResource(resource.data)) {
      limits[resource.type] = {
        maxValue: resource.data.max,
        regenRateSeconds: resource.data.regenRateSeconds,
        regenStepUnits: resource.data.regenStepUnits,
      };
    }
  }
  return limits;
}
