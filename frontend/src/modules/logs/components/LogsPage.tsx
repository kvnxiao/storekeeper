import { useQuery } from "@tanstack/solid-query";
import { createVirtualizer } from "@tanstack/solid-virtual";
import ArrowDown from "lucide-solid/icons/arrow-down";
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  Match,
  on,
  onMount,
  Show,
  Switch,
  type VoidComponent,
} from "solid-js";
import { core } from "@/modules/core/core.state";
import type { LogLevel } from "@/modules/logs/logs.constants";
import { logTailQueryOptions, openLogFolder } from "@/modules/logs/logs.query";
import {
  containsSelection,
  errorText,
  fieldSummary,
  filterEntries,
  isAtBottom,
  type LogEntry,
  logLevelOptions,
  parseLogLines,
  parseLogTimestamp,
} from "@/modules/logs/logs.utils";
import { Button } from "@/modules/ui/components/Button";
import { ErrorBanner } from "@/modules/ui/components/ErrorBanner";
import { Select } from "@/modules/ui/components/Select";
import { TextField } from "@/modules/ui/components/TextField";
import { cn } from "@/modules/ui/ui.styles";
import * as m from "@/paraglide/messages";

/** Height of one unwrapped row, used until the row has been measured. */
const ROW_ESTIMATE_PX = 28;

const ROW_OVERSCAN = 12;

const LEVEL_CLASS: Record<LogLevel, string> = {
  error: "bg-red-500/15 text-red-700 dark:bg-red-500/10 dark:text-red-400",
  warn: "bg-amber-400/20 text-amber-700 dark:bg-amber-400/10 dark:text-amber-400",
  info: "bg-zinc-600/10 text-zinc-700 dark:bg-white/5 dark:text-zinc-400",
  debug: "bg-sky-500/15 text-sky-700 dark:bg-sky-500/10 dark:text-sky-400",
  trace: "bg-zinc-500/10 text-zinc-500 dark:bg-white/5 dark:text-zinc-500",
};

// Fixed widths rather than intrinsic ones: each row is its own grid, so only a
// shared template lines the columns up down the list.
const ROW_COLUMNS =
  "grid grid-cols-[4.5rem_2.75rem_1fr_7rem] gap-x-3 sm:grid-cols-[4.5rem_2.75rem_1fr_13rem]";

const MUTED_TEXT = "text-zinc-500 dark:text-zinc-400";

/** Reads the stamp in the viewer's locale; a value that will not parse renders as written. */
function entryTime(entry: LogEntry): string {
  const stamped = parseLogTimestamp(entry.timestamp);
  return stamped ? core.timeWithSecondsFormatter().format(stamped) : entry.timestamp;
}

interface LogRowProps {
  entry: LogEntry;
}

const LogRow: VoidComponent<LogRowProps> = (props) => (
  <div
    class={cn(
      ROW_COLUMNS,
      "items-baseline border-b border-zinc-950/5 px-3 py-1 dark:border-white/5",
    )}
  >
    <span class={cn("tabular-nums", MUTED_TEXT)}>{entryTime(props.entry)}</span>
    <span
      class={cn(
        "rounded px-1 text-center text-[10px]/5 font-semibold uppercase",
        LEVEL_CLASS[props.entry.level],
      )}
    >
      {props.entry.level}
    </span>
    <span class="min-w-0 break-words text-zinc-950 dark:text-white">
      {props.entry.message}
      <Show when={fieldSummary(props.entry)}>
        {(summary) => <span class={MUTED_TEXT}> {summary()}</span>}
      </Show>
    </span>
    <span class="truncate text-right text-zinc-400 dark:text-zinc-500">{props.entry.target}</span>
  </div>
);

export const LogsPage: VoidComponent = () => {
  const tail = useQuery(() => logTailQueryOptions());

  const [minimumLevel, setMinimumLevel] = createSignal<LogLevel>("info");
  const [search, setSearch] = createSignal("");

  // Parsing is memoized apart from filtering so a keystroke re-filters the
  // entries instead of re-parsing every line of the tail.
  const entries = createMemo(() => parseLogLines(tail.data ?? []));
  const visible = createMemo(() => filterEntries(entries(), minimumLevel(), search()));

  const [scroller, setScroller] = createSignal<HTMLDivElement>();
  const [following, setFollowing] = createSignal(true);

  const virtualizer = createVirtualizer({
    get count() {
      return visible().length;
    },
    getScrollElement: () => scroller() ?? null,
    estimateSize: () => ROW_ESTIMATE_PX,
    overscan: ROW_OVERSCAN,
  });

  const scrollToLatest = (): void => {
    const count = visible().length;
    if (count > 0) {
      virtualizer.scrollToIndex(count - 1, { align: "end" });
    }
  };

  // Re-anchors on the total size as well as the count: a row measured after it
  // renders moves the bottom edge without adding an entry.
  createEffect(
    on([() => visible().length, () => virtualizer.getTotalSize()], () => {
      const element = scroller();
      if (element === undefined || !following() || containsSelection(element)) {
        return;
      }
      scrollToLatest();
    }),
  );

  return (
    <div class="flex h-screen flex-col p-4">
      <header class="mb-3 flex items-center justify-between gap-3">
        <div class="flex items-baseline gap-3">
          <h1 class="text-lg font-bold text-zinc-950 dark:text-white">{m.logs_window_title()}</h1>
          <span class={cn("text-sm tabular-nums", MUTED_TEXT)}>
            {m.logs_entry_count({ count: visible().length })}
          </span>
        </div>
        <Button color="light" onClick={() => openLogFolder()}>
          {m.logs_open_folder()}
        </Button>
      </header>

      <div class="mb-3 flex flex-wrap items-start gap-3">
        <div class="w-44">
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
            inputClass="h-9"
          />
        </div>
      </div>

      <Show when={tail.error}>
        {(error) => (
          <ErrorBanner class="mb-3">
            {m.logs_load_failed({ error: errorText(error()) })}
          </ErrorBanner>
        )}
      </Show>

      <div class="relative min-h-0 flex-1">
        {/* Held back until the node is in the document: a ref assigned during
            render still belongs to the template's contents document, whose
            defaultView is null, and the virtualizer that attaches to it never
            measures the container. */}
        <div
          ref={(node) => onMount(() => setScroller(node))}
          onScroll={(event) => setFollowing(isAtBottom(event.currentTarget))}
          class="h-full overflow-y-auto rounded-lg font-mono text-xs ring-1 ring-zinc-950/10 [overflow-anchor:none] dark:ring-white/10"
        >
          <Switch>
            <Match when={tail.isPending}>
              <p class={cn("p-3 font-sans text-sm", MUTED_TEXT)}>{m.logs_loading()}</p>
            </Match>
            <Match when={visible().length === 0}>
              <p class={cn("p-3 font-sans text-sm", MUTED_TEXT)}>{m.logs_empty()}</p>
            </Match>
            <Match when={visible().length > 0}>
              <div class="relative w-full" style={{ height: `${virtualizer.getTotalSize()}px` }}>
                <For each={virtualizer.getVirtualItems()}>
                  {(item) => (
                    <Show when={visible()[item.index]}>
                      {(entry) => (
                        <div
                          ref={(element) => {
                            element.dataset.index = String(item.index);
                            virtualizer.measureElement(element);
                          }}
                          class="absolute left-0 top-0 w-full"
                          style={{ transform: `translateY(${item.start}px)` }}
                        >
                          <LogRow entry={entry()} />
                        </div>
                      )}
                    </Show>
                  )}
                </For>
              </div>
            </Match>
          </Switch>
        </div>

        <div
          class={cn(
            "pointer-events-none absolute inset-x-0 bottom-3 flex justify-center",
            "transition-[opacity,scale] duration-150 ease-[cubic-bezier(0.2,0,0,1)] motion-reduce:transition-none",
            following() ? "scale-95 opacity-0" : "scale-100 opacity-100",
          )}
          aria-hidden={following()}
        >
          <Button
            color="light"
            class={following() ? "pointer-events-none" : "pointer-events-auto"}
            tabIndex={following() ? -1 : 0}
            onClick={() => {
              setFollowing(true);
              scrollToLatest();
            }}
          >
            <ArrowDown aria-hidden="true" class="size-4" />
            {m.logs_jump_to_latest()}
          </Button>
        </div>
      </div>
    </div>
  );
};
