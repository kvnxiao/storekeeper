import * as m from "@/paraglide/messages";
import "@formatjs/intl-durationformat/polyfill.js";

// Formatter cache keyed by locale — recreated only when locale changes
let cachedLocale: string | undefined;
let durationFormatter: Intl.DurationFormat;
let timeOnlyFormatter: Intl.DateTimeFormat;
let weekdayTimeFormatter: Intl.DateTimeFormat;

function getFormatters(locale: string) {
  if (locale !== cachedLocale) {
    cachedLocale = locale;
    durationFormatter = new Intl.DurationFormat(locale, {
      style: locale.startsWith("en") ? "narrow" : "short",
    });
    timeOnlyFormatter = new Intl.DateTimeFormat(locale, {
      hour: "numeric",
      minute: "2-digit",
    });
    weekdayTimeFormatter = new Intl.DateTimeFormat(locale, {
      weekday: "short",
      hour: "numeric",
      minute: "2-digit",
    });
  }
  return { durationFormatter, timeOnlyFormatter, weekdayTimeFormatter };
}

/**
 * Formats a datetime string to human-readable duration remaining.
 *
 * @param datetime - ISO 8601 datetime string for the target time
 * @param nowMs - Current time in milliseconds (from tick atom)
 * @param locale - Current app locale (drives formatter language)
 * @returns Formatted duration string like "2h 13m" or "Full"
 */
export function formatTimeRemaining(
  datetime: string | null | undefined,
  nowMs: number,
  locale: string,
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

  const { durationFormatter } = getFormatters(locale);
  return durationFormatter.format({
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
 * @param locale - Current app locale (drives formatter language)
 * @returns Formatted datetime like "3:17 PM" (today) or "Mon 3:17 PM" (other days), or null
 */
export function formatAbsoluteDateTime(
  datetime: string | null | undefined,
  nowMs: number,
  locale: string,
): string | null {
  if (!datetime) return null;
  const target = new Date(datetime);
  const now = new Date(nowMs);
  const isToday =
    target.getFullYear() === now.getFullYear() &&
    target.getMonth() === now.getMonth() &&
    target.getDate() === now.getDate();
  const { timeOnlyFormatter, weekdayTimeFormatter } = getFormatters(locale);
  return isToday
    ? timeOnlyFormatter.format(target)
    : weekdayTimeFormatter.format(target);
}
