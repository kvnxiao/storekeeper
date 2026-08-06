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
 * or "Full" once it has passed. `nowMs` comes from the tick signal so every
 * countdown on screen measures against the same instant.
 */
export function formatTimeRemaining(
  datetime: string | null | undefined,
  nowMs: number,
  durationFmt: Intl.DurationFormat,
): string {
  const targetMs = datetime ? new Date(datetime).getTime() : Number.NaN;
  const totalSeconds = Math.floor((targetMs - nowMs) / 1000);
  if (Number.isNaN(totalSeconds) || totalSeconds <= 0) {
    return m.time_remaining_full();
  }
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

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
export function isPastDateTime(datetime: string | null | undefined, nowMs: number): boolean {
  const targetMs = datetime ? new Date(datetime).getTime() : Number.NaN;
  return Number.isNaN(targetMs) || targetMs <= nowMs;
}

/**
 * Formats an ISO 8601 `datetime` as a clock time ("3:17 PM"), prefixed with the
 * weekday when it does not fall on the same calendar day as `nowMs`. Returns
 * null when there is nothing parseable to format.
 */
export function formatAbsoluteDateTime(
  datetime: string | null | undefined,
  nowMs: number,
  timeOnlyFmt: Intl.DateTimeFormat,
  weekdayTimeFmt: Intl.DateTimeFormat,
): string | null {
  if (!datetime) {
    return null;
  }
  const target = new Date(datetime);
  if (Number.isNaN(target.getTime())) {
    return null;
  }
  const now = new Date(nowMs);
  const isToday =
    target.getFullYear() === now.getFullYear() &&
    target.getMonth() === now.getMonth() &&
    target.getDate() === now.getDate();
  return isToday ? timeOnlyFmt.format(target) : weekdayTimeFmt.format(target);
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
      };
    }
  }
  return limits;
}
