import { queryOptions } from "@tanstack/solid-query";
import { invoke } from "@tauri-apps/api/core";

/** Interval between log-tail reads while the viewer is open. */
const LOG_POLL_MS = 2_000;

/** Maximum number of trailing entries requested per read. */
const LOG_TAIL_LINES = 1_000;

/**
 * Query options for the tail of the current day's log file, as raw JSON lines.
 *
 * TanStack's structural sharing keeps the previous array when a poll reads the
 * same lines, so an idle viewer does not rebuild every row.
 */
export function logTailQueryOptions() {
  return queryOptions({
    queryKey: ["log-tail"],
    queryFn: async () => invoke<string[]>("read_log_tail", { lines: LOG_TAIL_LINES }),
    refetchInterval: LOG_POLL_MS,
  });
}

/** Open the log directory in the OS file manager. */
export function openLogFolder(): void {
  invoke("open_log_folder").catch(console.error);
}

/** Open the log viewer window, or focus the one already open. */
export function openLogsWindow(): void {
  invoke("open_logs_window").catch(console.error);
}
