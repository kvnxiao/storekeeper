import { render, screen } from "@solidjs/testing-library";
import userEvent from "@testing-library/user-event";
import { QueryClientProvider, useMutation } from "@tanstack/solid-query";
import type { VoidComponent } from "solid-js";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vite-plus/test";
import { queryClient } from "@/modules/core/core.queryClient";
import { core } from "@/modules/core/core.state";
import { DashboardContent } from "@/modules/dashboard/components/DashboardContent";
import { GameId } from "@/modules/games/games.types";
import { refreshResourcesMutationOptions } from "@/modules/resources/resources.query";
import type { AllResources } from "@/modules/resources/resources.types";

const listeners = new Map<string, (event: { payload: unknown }) => void>();

/** Resin value the command response carries; never rendered on its own. */
const RESPONSE_RESIN = 180;

/** How far back a snapshot's fetch time sits, keeping the render gap non-zero. */
const FETCH_LEAD_MS = 1_000;

const FULL_IN_MS = 3_600_000;
const COOLDOWN_MS = 5 * 86_400_000;
const WAIT_TIMEOUT_MS = 3_000;

function snapshot(resin: number): AllResources {
  // Backdated so the fetch-to-render gap this file asserts on stays non-zero
  // when a render lands in the same millisecond the snapshot was built.
  const now = Date.now() - FETCH_LEAD_MS;
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

/** Resolves once the rendered text matches, instead of sleeping on a guess. */
const showsText = (container: HTMLElement, text: string) =>
  vi.waitFor(() => expect(container.textContent).toContain(text), {
    timeout: WAIT_TIMEOUT_MS,
  });

/** One icon per card shell, so a re-created card shows up as a new node. */
const cardIcons = (container: HTMLElement) => [...container.querySelectorAll("img")];

/** Stands in for the refresh button, which lives on the route's header. */
const RefreshTrigger: VoidComponent = () => {
  const refresh = useMutation(() => refreshResourcesMutationOptions());

  return (
    <button type="button" onClick={() => refresh.mutate()}>
      trigger refresh
    </button>
  );
};

/** The rendered game section, which a re-created gating chain would replace. */
const genshinSection = () => screen.getByRole("button", { name: "Genshin Impact" });

async function mount() {
  const view = render(() => (
    <QueryClientProvider client={queryClient}>
      <DashboardContent />
      <RefreshTrigger />
    </QueryClientProvider>
  ));
  await showsText(view.container, "100/200");
  return view;
}

describe("resource refresh", () => {
  beforeAll(() => core.initDashboard());

  beforeEach(() => queryClient.clear());

  it("updates the cards in place rather than re-creating them", async () => {
    const { container } = await mount();
    const cards = cardIcons(container);
    const section = genshinSection();
    expect(cards).toHaveLength(4);

    emit("refresh-started");
    emit("resources-updated", snapshot(120));
    await showsText(container, "120/200");

    expect(genshinSection()).toBe(section);
    for (const [index, card] of cardIcons(container).entries()) {
      expect(card, `card ${index} was re-created`).toBe(cards[index]);
    }
  });

  it("leaves the view to the snapshot event, not the refresh command response", async () => {
    const { container } = await mount();

    await userEvent.setup().click(screen.getByRole("button", { name: "trigger refresh" }));
    await vi.waitFor(() => expect(queryClient.isMutating()).toBe(0), {
      timeout: WAIT_TIMEOUT_MS,
    });

    expect(container.textContent).toContain("100/200");
    expect(container.textContent).not.toContain(`${RESPONSE_RESIN}/200`);

    emit("resources-updated", snapshot(RESPONSE_RESIN));
    await showsText(container, `${RESPONSE_RESIN}/200`);
  });

  it("unmasks a game's cards on its own event, without waiting for the refresh to end", async () => {
    const { container } = await mount();
    const masked = () => container.querySelectorAll(".mask-shimmer").length;
    // The snapshot carries two of Genshin's four resources, so the other two
    // cards stay masked for want of data whatever the refresh is doing.
    const withoutData = masked();

    emit("refresh-started");
    await vi.waitFor(() => expect(masked()).toBe(4), { timeout: WAIT_TIMEOUT_MS });

    emit("game-resource-updated", {
      gameId: GameId.GenshinImpact,
      data: snapshot(140).games?.[GameId.GenshinImpact],
    });
    await vi.waitFor(() => expect(masked()).toBe(withoutData), { timeout: WAIT_TIMEOUT_MS });

    emit("resources-updated", snapshot(140));
    await showsText(container, "140/200");
  });

  // A snapshot's cooldown deadline is `fetch time + 5d`, so a clock older than
  // the fetch renders the full 5d instead of the 4d 23h 59m left. The gap
  // between the fetch and the render is the whole signal, so this file has to
  // run on the real clock: a frozen one makes that gap 0 and renders "5d".
  it("measures a cooldown against the clock the snapshot arrived on", async () => {
    const { container } = await mount();
    expect(container.textContent).toContain("4d 23h 59m");
    expect(container.textContent).not.toContain("in 5d");

    emit("resources-updated", snapshot(120));
    await showsText(container, "120/200");

    expect(container.textContent).toContain("4d 23h 59m");
    expect(container.textContent).not.toContain("in 5d");
  });
});
