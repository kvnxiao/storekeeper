import { useQuery } from "@tanstack/solid-query";
import { createFileRoute } from "@tanstack/solid-router";
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  Match,
  on,
  Show,
  Switch,
  type VoidComponent,
} from "solid-js";
import { core } from "@/modules/core/core.state";
import type { LogLevel } from "@/modules/logs/logs.constants";
import { logTailQueryOptions, openLogFolder } from "@/modules/logs/logs.query";
import {
  errorText,
  fieldSummary,
  filterEntries,
  type LogEntry,
  logLevelOptions,
  parseLogLines,
  parseLogTimestamp,
} from "@/modules/logs/logs.utils";
import { Badge, type BadgeProps } from "@/modules/ui/components/Badge";
import { Button } from "@/modules/ui/components/Button";
import { ErrorBanner } from "@/modules/ui/components/ErrorBanner";
import { Select } from "@/modules/ui/components/Select";
import { Switch as SwitchControl } from "@/modules/ui/components/Switch";
import { TextField } from "@/modules/ui/components/TextField";
import * as m from "@/paraglide/messages";

const LEVEL_VARIANTS: Record<LogLevel, BadgeProps["variant"]> = {
  error: "destructive",
  warn: "warning",
  info: "default",
  debug: "secondary",
  trace: "outline",
};

/** Reads the stamp in the viewer's locale; a value that will not parse renders as written. */
function entryTime(entry: LogEntry): string {
  const stamped = parseLogTimestamp(entry.timestamp);
  return stamped ? core.timeWithSecondsFormatter().format(stamped) : entry.timestamp;
}

const LogsPage: VoidComponent = () => {
  const tail = useQuery(() => logTailQueryOptions());

  const [minimumLevel, setMinimumLevel] = createSignal<LogLevel>("info");
  const [search, setSearch] = createSignal("");
  const [followTail, setFollowTail] = createSignal(true);

  // Parsing is memoized apart from filtering so a keystroke re-filters the
  // entries instead of re-parsing every line of the tail.
  const entries = createMemo(() => parseLogLines(tail.data ?? []));
  const visible = createMemo(() => filterEntries(entries(), minimumLevel(), search()));

  const [list, setList] = createSignal<HTMLDivElement>();

  // Newest entries render last, so following the tail means scrolling to the bottom.
  createEffect(
    on([visible, followTail, list], ([, follow, element]) => {
      if (follow && element) {
        element.scrollTop = element.scrollHeight;
      }
    }),
  );

  return (
    <div class="flex h-screen flex-col p-4">
      <header class="mb-3 flex items-center justify-between gap-3">
        <div class="flex items-baseline gap-3">
          <h1 class="text-lg font-bold text-zinc-950 dark:text-white">{m.logs_window_title()}</h1>
          <span class="text-sm text-zinc-500 dark:text-zinc-400">
            {m.logs_entry_count({ count: visible().length })}
          </span>
        </div>
        <Button color="light" onClick={() => openLogFolder()}>
          {m.logs_open_folder()}
        </Button>
      </header>

      <div class="mb-3 flex flex-wrap items-end gap-4">
        <div class="w-40">
          <Select
            label={m.logs_level_filter()}
            value={minimumLevel()}
            onChange={setMinimumLevel}
            options={logLevelOptions()}
          />
        </div>
        <div class="min-w-60 flex-1">
          <TextField
            label={m.logs_search()}
            placeholder={m.logs_search_placeholder()}
            value={search()}
            onChange={setSearch}
          />
        </div>
        <SwitchControl checked={followTail()} onChange={setFollowTail}>
          {m.logs_follow_tail()}
        </SwitchControl>
      </div>

      <Show when={tail.error}>
        {(error) => (
          <ErrorBanner class="mb-3">
            {m.logs_load_failed({ error: errorText(error()) })}
          </ErrorBanner>
        )}
      </Show>

      <div
        ref={setList}
        class="min-h-0 flex-1 overflow-y-auto rounded-lg font-mono text-xs ring-1 ring-zinc-950/10 dark:ring-white/10"
      >
        <Switch>
          <Match when={tail.isPending}>
            <p class="p-3 font-sans text-sm text-zinc-500 dark:text-zinc-400">{m.logs_loading()}</p>
          </Match>
          <Match when={visible().length === 0}>
            <p class="p-3 font-sans text-sm text-zinc-500 dark:text-zinc-400">{m.logs_empty()}</p>
          </Match>
          <Match when={visible().length > 0}>
            <For each={visible()}>
              {(entry) => (
                <div class="flex items-start gap-2 border-b border-zinc-950/5 px-3 py-1.5 last:border-b-0 dark:border-white/5">
                  <span class="shrink-0 tabular-nums text-zinc-500 dark:text-zinc-400">
                    {entryTime(entry)}
                  </span>
                  <Badge variant={LEVEL_VARIANTS[entry.level]} class="shrink-0 uppercase">
                    {entry.level}
                  </Badge>
                  <span class="min-w-0 flex-1 break-words text-zinc-950 dark:text-white">
                    {entry.message}
                    <Show when={fieldSummary(entry)}>
                      {(summary) => (
                        <span class="text-zinc-500 dark:text-zinc-400"> {summary()}</span>
                      )}
                    </Show>
                  </span>
                  <span class="hidden shrink-0 text-zinc-400 sm:inline dark:text-zinc-500">
                    {entry.target}
                  </span>
                </div>
              )}
            </For>
          </Match>
        </Switch>
      </div>
    </div>
  );
};

export const Route = createFileRoute("/logs")({
  component: LogsPage,
});
