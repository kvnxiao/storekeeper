import { render, screen, within } from "@solidjs/testing-library";
import { QueryClientProvider } from "@tanstack/solid-query";
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  RouterProvider,
} from "@tanstack/solid-router";
import { beforeEach, describe, expect, it, vi } from "vite-plus/test";
import { queryClient } from "@/modules/core/core.queryClient";
import type { AppConfig, SecretsConfig } from "@/modules/settings/settings.types";
import { SettingsPage } from "@/modules/settings/components/SettingsPage";

/** Use a distinct UID per game so a crossed field path is visible. */
const CONFIG: AppConfig = {
  general: {
    poll_interval_secs: 300,
    start_minimized: false,
    log_level: "info",
    language: null,
    autostart: false,
  },
  games: {
    genshin_impact: { enabled: true, uid: "genshin-uid", auto_claim_daily_rewards: false },
    honkai_star_rail: { enabled: true, uid: "hsr-uid", auto_claim_daily_rewards: false },
    zenless_zone_zero: { enabled: true, uid: "zzz-uid", auto_claim_daily_rewards: false },
    wuthering_waves: { enabled: true, uid: "wuwa-uid" },
  },
};

const SECRETS: SecretsConfig = {
  hoyolab: { ltuid_v2: "", ltoken_v2: "", ltmid_v2: "" },
  kuro: { oauth_code: "" },
};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (command: string) => {
    switch (command) {
      case "get_config":
        return CONFIG;
      case "get_secrets":
        return SECRETS;
      case "get_all_resources":
        return { games: {} };
      default:
        return {};
    }
  }),
}));

// Mock the event bridge because SettingsPage initializes dashboard listeners.
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(async () => vi.fn()) }));

// Mount under a bare root because the generated route tree includes the
// document's <html> element.
async function renderSettingsPage(): Promise<void> {
  const rootRoute = createRootRoute();
  const router = createRouter({
    routeTree: rootRoute.addChildren([
      createRoute({ getParentRoute: () => rootRoute, path: "/", component: SettingsPage }),
    ]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  });

  render(() => (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  ));
  await screen.findByRole("heading", { level: 1, name: "Settings" });
}

function gameSection(title: string): HTMLElement {
  const card = screen.getByRole("heading", { level: 2, name: title }).closest("section");
  if (!(card instanceof HTMLElement)) {
    throw new Error(`no settings card for ${title}`);
  }
  return card;
}

describe("SettingsPage", () => {
  beforeEach(() => queryClient.clear());

  it("binds every game section to its own config entry", async () => {
    await renderSettingsPage();

    expect(within(gameSection("Genshin Impact")).getByLabelText("UID")).toHaveValue("genshin-uid");
    expect(within(gameSection("Honkai: Star Rail")).getByLabelText("UID")).toHaveValue("hsr-uid");
    expect(within(gameSection("Zenless Zone Zero")).getByLabelText("UID")).toHaveValue("zzz-uid");
    expect(within(gameSection("Wuthering Waves")).getByLabelText("UID")).toHaveValue("wuwa-uid");
  });

  it("gives the auto-claim toggle to the HoYoLab games only", async () => {
    await renderSettingsPage();

    expect(
      within(gameSection("Genshin Impact")).getByText("Auto-claim daily rewards"),
    ).toBeInTheDocument();
    expect(
      within(gameSection("Wuthering Waves")).queryByText("Auto-claim daily rewards"),
    ).toBeNull();
  });
});
