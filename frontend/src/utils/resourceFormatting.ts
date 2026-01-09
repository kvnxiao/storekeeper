import type { ProgressBarProps } from "@/components/ui/ProgressBar";

type ProgressColor = NonNullable<ProgressBarProps["color"]>;

/** Formats a datetime string to human-readable time remaining */
export function formatTime(datetime: string | null | undefined): string {
  if (!datetime) return "Full";

  const target = new Date(datetime);
  const now = new Date();
  const diffMs = target.getTime() - now.getTime();

  if (diffMs <= 0) return "Full";

  const seconds = Math.floor(diffMs / 1000);
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);

  if (hours > 0) return `${hours}h ${mins}m`;
  return `${mins}m`;
}

/** Checks if a datetime is in the past */
export function isPastDateTime(datetime: string | null | undefined): boolean {
  if (!datetime) return true;
  const target = new Date(datetime);
  const now = new Date();
  return target.getTime() <= now.getTime();
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
