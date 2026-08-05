import type { GameId } from "@/modules/games/games.types";
import * as m from "@/paraglide/messages";
import {
  type AllResources,
  isStaminaResource,
  type ResourceLimits,
} from "@/modules/resources/resources.types";

/**
 * Formats a datetime string to human-readable duration remaining.
 *
 * @param datetime - ISO 8601 datetime string for the target time
 * @param nowMs - Current time in milliseconds (from the tick signal)
 * @param durationFmt - Intl.DurationFormat instance for the current locale
 * @returns Formatted duration string like "2h 13m" or "Full"
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
    seconds: hours === 0 && seconds > 0 ? seconds : undefined,
  });
}

/**
 * Checks if a datetime is in the past. Missing or unparseable datetimes count
 * as past.
 *
 * @param datetime - ISO 8601 datetime string
 * @param nowMs - Current time in milliseconds (from the tick signal)
 */
export function isPastDateTime(datetime: string | null | undefined, nowMs: number): boolean {
  const targetMs = datetime ? new Date(datetime).getTime() : Number.NaN;
  return Number.isNaN(targetMs) || targetMs <= nowMs;
}

/**
 * Formats a datetime string to absolute date/time.
 * Shows weekday when the target date is not today.
 *
 * @param datetime - ISO 8601 datetime string
 * @param nowMs - Current time in milliseconds (from the tick signal)
 * @param timeOnlyFmt - Intl.DateTimeFormat for time-only display
 * @param weekdayTimeFmt - Intl.DateTimeFormat for weekday + time display
 * @returns Formatted datetime like "3:17 PM" (today) or "Mon 3:17 PM" (other days), or null
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
): Partial<Record<string, ResourceLimits>> {
  const limits: Record<string, ResourceLimits> = {};
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
