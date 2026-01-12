import "@formatjs/intl-durationformat/polyfill.js";

import type { ProgressBarProps } from "@/modules/ui/components/ProgressBar";

type ProgressColor = NonNullable<ProgressBarProps["color"]>;

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
  if (!datetime) return "Full";

  const targetMs = new Date(datetime).getTime();
  const diffMs = targetMs - nowMs;

  if (diffMs <= 0) return "Full";

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
 * @deprecated Use formatTimeRemaining with tick atom instead
 */
export function formatTime(datetime: string | null | undefined): string {
  return formatTimeRemaining(datetime, Date.now());
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
  return targetMs <= nowMs;
}

/** Maps resource type to display name */
export function getResourceDisplayName(type: string): string {
  const names: Record<string, string> = {
    resin: "Original Resin",
    parametric_transformer: "Transformer",
    realm_currency: "Realm Currency",
    expeditions: "Expeditions",
    trailblaze_power: "Trailblaze Power",
    battery: "Battery",
    waveplates: "Waveplates",
  };
  return names[type] ?? type;
}

/** Get progress bar color variant based on percentage */
export function getProgressColor(
  percentage: number,
  isFull: boolean,
): ProgressColor {
  if (isFull) return "success";
  if (percentage >= 50) return "info";
  if (percentage >= 25) return "warning";
  return "danger";
}
