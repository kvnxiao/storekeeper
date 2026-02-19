import * as m from "@/paraglide/messages";
import "@formatjs/intl-durationformat/polyfill.js";

/**
 * Formats a datetime string to human-readable duration remaining.
 *
 * @param datetime - ISO 8601 datetime string for the target time
 * @param nowMs - Current time in milliseconds (from tick atom)
 * @param durationFmt - Intl.DurationFormat instance for the current locale
 * @returns Formatted duration string like "2h 13m" or "Full"
 */
export function formatTimeRemaining(
  datetime: string | null | undefined,
  nowMs: number,
  durationFmt: Intl.DurationFormat,
): string {
  if (!datetime) return m.time_remaining_full();

  const targetMs = new Date(datetime).getTime();
  if (Number.isNaN(targetMs)) return m.time_remaining_full();

  const diffMs = targetMs - nowMs;

  if (diffMs <= 0) return m.time_remaining_full();

  const totalSeconds = Math.floor(diffMs / 1000);
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return durationFmt.format({
    days: days > 0 ? days : undefined,
    hours: days > 0 || hours > 0 ? hours : undefined,
    minutes: days > 0 || hours > 0 ? minutes : undefined,
    seconds: days === 0 && hours === 0 ? seconds : undefined,
  });
}

/**
 * Checks if a datetime is in the past.
 *
 * @param datetime - ISO 8601 datetime string
 * @param nowMs - Current time in milliseconds (from tick atom)
 */
export function isPastDateTime(
  datetime: string | null | undefined,
  nowMs: number,
): boolean {
  if (!datetime) return true;
  const targetMs = new Date(datetime).getTime();
  if (Number.isNaN(targetMs)) return true;
  return targetMs <= nowMs;
}

/**
 * Formats a datetime string to absolute date/time.
 * Shows weekday when the target date is not today.
 *
 * @param datetime - ISO 8601 datetime string
 * @param nowMs - Current time in milliseconds (from tick atom)
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
  if (!datetime) return null;
  const target = new Date(datetime);
  const now = new Date(nowMs);
  const isToday =
    target.getFullYear() === now.getFullYear() &&
    target.getMonth() === now.getMonth() &&
    target.getDate() === now.getDate();
  return isToday ? timeOnlyFmt.format(target) : weekdayTimeFmt.format(target);
}
