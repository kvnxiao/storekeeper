import * as m from "@/paraglide/messages";
import "@formatjs/intl-durationformat/polyfill.js";

const durationFormatter = new Intl.DurationFormat(undefined, {
  style: "narrow",
});

/**
 * Formats a datetime string to human-readable duration remaining.
 *
 * @param datetime - ISO 8601 datetime string for the target time
 * @param nowMs - Current time in milliseconds (from tick atom)
 * @returns Formatted duration string like "2h 13m" or "Full"
 */
export function formatTimeRemaining(
  datetime: string | null | undefined,
  nowMs: number,
): string {
  if (!datetime) return m.time_remaining_full();

  const targetMs = new Date(datetime).getTime();
  if (Number.isNaN(targetMs)) return m.time_remaining_full();

  const diffMs = targetMs - nowMs;

  if (diffMs <= 0) return m.time_remaining_full();

  const totalSeconds = Math.floor(diffMs / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return durationFormatter.format({
    hours: hours > 0 ? hours : undefined,
    minutes: minutes > 0 || hours > 0 ? minutes : undefined,
    seconds: hours === 0 && minutes === 0 ? seconds : undefined,
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

const absoluteDateFormatter = new Intl.DateTimeFormat(undefined, {
  weekday: "long",
  hour: "numeric",
  minute: "2-digit",
});

/**
 * Formats a datetime string to absolute date/time.
 *
 * @param datetime - ISO 8601 datetime string
 * @returns Formatted datetime like "Tuesday 12:43 pm" or null if no datetime
 */
export function formatAbsoluteDateTime(
  datetime: string | null | undefined,
): string | null {
  if (!datetime) return null;
  return absoluteDateFormatter.format(new Date(datetime));
}
