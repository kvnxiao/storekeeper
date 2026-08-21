/**
 * Parse and filter the backend's JSON log file.
 *
 * The backend writes one `tracing_subscriber` JSON object per line. Parse lines
 * independently so a truncated tail drops one entry rather than the whole read.
 */

import { LOG_LEVELS, type LogLevel } from "@/modules/logs/logs.constants";
import type { SelectOption } from "@/modules/ui/components/Select";
import * as m from "@/paraglide/messages";

export interface LogEntry {
  /** RFC 3339 timestamp as the backend wrote it. */
  timestamp: string;
  level: LogLevel;
  /** Emitting module path, e.g. `storekeeper_app_tauri::scheduled_claim`. */
  target: string;
  message: string;
  /** Structured fields excluding `message`. */
  fields: Record<string, unknown>;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function asLogLevel(value: unknown): LogLevel | null {
  if (typeof value !== "string") {
    return null;
  }
  const lowered = value.toLowerCase();
  return LOG_LEVELS.find((level) => level === lowered) ?? null;
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

/**
 * Parses one log line into a `LogEntry`.
 *
 * A tail read landing mid-write yields a truncated line, so a line that is not
 * JSON or carries no recognizable level returns `null` instead of throwing.
 */
export function parseLogLine(line: string): LogEntry | null {
  let raw: unknown;
  try {
    raw = JSON.parse(line);
  } catch {
    return null;
  }

  if (!isRecord(raw)) {
    return null;
  }

  const level = asLogLevel(raw.level);
  if (level === null) {
    return null;
  }

  const fields = isRecord(raw.fields) ? raw.fields : {};
  const { message, ...rest } = fields;

  return {
    timestamp: asString(raw.timestamp),
    level,
    target: asString(raw.target),
    message: asString(message),
    fields: rest,
  };
}

export function parseLogLines(lines: readonly string[]): LogEntry[] {
  return lines.map((line) => parseLogLine(line)).filter((entry) => entry !== null);
}

export function admitsLevel(minimum: LogLevel, level: LogLevel): boolean {
  return LOG_LEVELS.indexOf(level) <= LOG_LEVELS.indexOf(minimum);
}

function matchesSearch(entry: LogEntry, needle: string): boolean {
  return (
    entry.message.toLowerCase().includes(needle) ||
    entry.target.toLowerCase().includes(needle) ||
    JSON.stringify(entry.fields).toLowerCase().includes(needle)
  );
}

/** Keep entries at or above `minimum` whose text contains `search`. */
export function filterEntries(
  entries: readonly LogEntry[],
  minimum: LogLevel,
  search: string,
): LogEntry[] {
  const needle = search.trim().toLowerCase();
  return entries.filter(
    (entry) => admitsLevel(minimum, entry.level) && (needle === "" || matchesSearch(entry, needle)),
  );
}

/**
 * Build localized log-level choices in descending severity order.
 *
 * Evaluated at call time so labels follow the active locale.
 */
export function logLevelOptions(): SelectOption<LogLevel>[] {
  return [
    { id: "error", label: m.log_level_error() },
    { id: "warn", label: m.log_level_warn() },
    { id: "info", label: m.log_level_info() },
    { id: "debug", label: m.log_level_debug() },
    { id: "trace", label: m.log_level_trace() },
  ];
}

export function fieldSummary(entry: LogEntry): string {
  return Object.entries(entry.fields)
    .map(([name, value]) => `${name}=${typeof value === "string" ? value : JSON.stringify(value)}`)
    .join(" ");
}

/** Pixel slack that still counts a scroll position as the bottom edge. */
const BOTTOM_SLACK_PX = 24;

export interface ScrollMetrics {
  scrollHeight: number;
  scrollTop: number;
  clientHeight: number;
}

/** Treat a scroll position within the configured slack as the bottom edge. */
export function isAtBottom(metrics: ScrollMetrics): boolean {
  return metrics.scrollHeight - metrics.scrollTop - metrics.clientHeight <= BOTTOM_SLACK_PX;
}

/**
 * Report whether the document's selection starts inside `element`.
 *
 * Stop tail-following while text inside a virtualized row is selected because
 * scrolling recycles that row's DOM node.
 */
export function containsSelection(element: Node): boolean {
  const selection = document.getSelection();
  if (selection === null || selection.isCollapsed) {
    return false;
  }
  const anchor = selection.anchorNode;
  return anchor !== null && element.contains(anchor);
}

/** Read a rejected Tauri command's message before stringifying the value. */
export function errorText(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String(error.message);
  }
  return String(error);
}
