/**
 * Parsing and filtering for the backend's JSON log file.
 *
 * The backend writes one `tracing_subscriber` JSON object per line, so the
 * viewer parses each line on its own and a truncated tail costs one entry
 * rather than the whole read.
 */

import { LOG_LEVELS, type LogLevel } from "@/modules/logs/logs.constants";
import type { SelectOption } from "@/modules/ui/components/Select";
import * as m from "@/paraglide/messages";

/** One parsed line of the log file. */
export interface LogEntry {
  /** RFC 3339 timestamp as the backend wrote it. */
  timestamp: string;
  level: LogLevel;
  /** Emitting module path, e.g. `storekeeper_app_tauri::scheduled_claim`. */
  target: string;
  message: string;
  /** Structured fields other than the message. */
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

/** Parses every line, dropping those that are not a log record. */
export function parseLogLines(lines: readonly string[]): LogEntry[] {
  return lines.map((line) => parseLogLine(line)).filter((entry) => entry !== null);
}

/** Reports whether `level` is at least as severe as `minimum`. */
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

/** Keeps the entries at or above `minimum` whose text contains `search`. */
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
 * Reads an entry's timestamp as a `Date`.
 *
 * The backend stamps UTC, so the viewer has to convert before it can show a
 * time that lines up with the rest of the app. A value that is not a date
 * returns `null`, and the caller renders it verbatim.
 */
export function parseLogTimestamp(timestamp: string): Date | null {
  const parsed = new Date(timestamp);
  return Number.isNaN(parsed.getTime()) ? null : parsed;
}

/**
 * Log-level choices for a `Select`, most to least severe.
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

/** Renders an entry's structured fields as `name=value` pairs. */
export function fieldSummary(entry: LogEntry): string {
  return Object.entries(entry.fields)
    .map(([name, value]) => `${name}=${typeof value === "string" ? value : JSON.stringify(value)}`)
    .join(" ");
}

/** Reads a rejected `invoke`'s message; a `CommandError` stringifies to `[object Object]`. */
export function errorText(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String(error.message);
  }
  return String(error);
}
