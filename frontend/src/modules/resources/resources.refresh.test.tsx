import { render } from "@solidjs/testing-library";
import { QueryClientProvider, useMutation, useQuery } from "@tanstack/solid-query";
import { createMemo, Show } from "solid-js";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vite-plus/test";
import { queryClient } from "@/modules/core/core.queryClient";
import { core } from "@/modules/core/core.state";
import { GameId } from "@/modules/games/games.types";
import { GenshinSection } from "@/modules/games/genshin/components/GenshinSection";
import { refreshResourcesMutationOptions } from "@/modules/resources/resources.query";
import type { AllResources } from "@/modules/resources/resources.types";
import { configQueryOptions } from "@/modules/settings/settings.query";
import { enabledGamesFromConfig } from "@/modules/settings/settings.utils";

const listeners = new Map<string, (event: { payload: unknown }) => void>();

/** Resin value the command response carries; never rendered on its own. */
const RESPONSE_RESIN = 180;

const FULL_IN_MS = 3_600_000;
const COOLDOWN_MS = 5 * 86_400_000;

function snapshot(resin: number): AllResources {
  const now = Date.now();
  return {
    games: {
      [GameId.GenshinImpact]: [
        {
          type: "resin",
          data: { current: resin, max: 200, fullAt: iso(now + FULL_IN_MS), regenRateSeconds: 480 },
        },
        {
          type: "parametric_transformer",
          data: { isReady: false, readyAt: iso(now + COOLDOWN_MS) },
        },
      ],
    },
    lastUpdated: iso(now),
  };
}

function iso(ms: number): string {
  return new Date(ms).toISOString();
}

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (command: string) => {
    switch (command) {
      case "get_config":
        return { games: { genshin_impact: { enabled: true } } };
      case "get_all_resources":
        return snapshot(100);
      case "refresh_resources":
        return snapshot(RESPONSE_RESIN);
      case "get_effective_locale":
        return "en";
      default:
        return {};
    }
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, handler: (event: { payload: unknown }) => void) => {
    listeners.set(name, handler);
    return () => listeners.delete(name);
  }),
}));

function emit(name: string, payload?: unknown): void {
  listeners.get(name)?.({ payload });
}

const settled = () => new Promise((resolve) => setTimeout(resolve, 20));

/** One icon per card shell, so a re-created card shows up as a new node. */
const cardIcons = (container: HTMLElement) => [...container.querySelectorAll("img")];

let triggerRefresh: () => void;

/** The dashboard's gating chain around one game section. */
const Dashboard = () => {
  const configQuery = useQuery(() => configQueryOptions());
  const refresh = useMutation(() => refreshResourcesMutationOptions());
  const enabledGames = createMemo(() => enabledGamesFromConfig(configQuery.data));

  triggerRefresh = () => refresh.mutate();

  return (
    <Show when={!configQuery.isPending}>
      <Show when={enabledGames().size > 0} fallback={<div>no games</div>}>
        <div data-testid="sections">
          <Show when={enabledGames().has(GameId.GenshinImpact)}>
            <GenshinSection />
          </Show>
        </div>
      </Show>
    </Show>
  );
};

async function mount() {
  const view = render(() => (
    <QueryClientProvider client={queryClient}>
      <Dashboard />
    </QueryClientProvider>
  ));
  await settled();
  return view;
}

describe("resource refresh", () => {
  beforeAll(() => core.init());

  beforeEach(async () => {
    queryClient.clear();
    await settled();
  });

  it("updates the cards in place rather than re-creating them", async () => {
    const { container } = await mount();
    const cards = cardIcons(container);
    const sections = container.querySelector("[data-testid=sections]");
    expect(cards).toHaveLength(4);
    expect(container.textContent).toContain("100/200");

    emit("refresh-started");
    await settled();
    emit("resources-updated", snapshot(120));
    await settled();

    expect(container.textContent).toContain("120/200");
    expect(container.querySelector("[data-testid=sections]")).toBe(sections);
    for (const [index, card] of cardIcons(container).entries()) {
      expect(card, `card ${index} was re-created`).toBe(cards[index]);
    }
  });

  it("leaves the view to the snapshot event, not the refresh command response", async () => {
    const { container } = await mount();
    expect(container.textContent).toContain("100/200");

    triggerRefresh();
    await settled();

    expect(container.textContent).toContain("100/200");
    expect(container.textContent).not.toContain(`${RESPONSE_RESIN}/200`);

    emit("resources-updated", snapshot(RESPONSE_RESIN));
    await settled();

    expect(container.textContent).toContain(`${RESPONSE_RESIN}/200`);
  });

  // A snapshot's cooldown deadline is `fetch time + 5d`, so a clock older than
  // the fetch renders the full 5d instead of the 4d 23h 59m left.
  it("measures a cooldown against the clock the snapshot arrived on", async () => {
    const { container } = await mount();
    expect(container.textContent).toContain("4d 23h 59m");
    expect(container.textContent).not.toContain("in 5d");

    emit("resources-updated", snapshot(120));
    await settled();

    expect(container.textContent).toContain("4d 23h 59m");
    expect(container.textContent).not.toContain("in 5d");
  });
});
