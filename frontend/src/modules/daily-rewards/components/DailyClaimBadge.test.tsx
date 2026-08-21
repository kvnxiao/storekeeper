import { render, screen } from "@solidjs/testing-library";
import { QueryClientProvider } from "@tanstack/solid-query";
import { beforeEach, describe, expect, it, vi } from "vite-plus/test";
import { queryClient } from "@/modules/core/core.queryClient";
import { DailyClaimBadge } from "@/modules/daily-rewards/components/DailyClaimBadge";
import { GameId } from "@/modules/games/games.types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (command: string) => {
    switch (command) {
      case "get_daily_reward_status":
        return { games: { GENSHIN_IMPACT: { info: { is_signed: false } } } };
      default:
        return {};
    }
  }),
}));

async function mountUnclaimed(): Promise<HTMLElement> {
  render(() => (
    <QueryClientProvider client={queryClient}>
      <DailyClaimBadge gameId={GameId.GenshinImpact} gameName="Genshin Impact" />
    </QueryClientProvider>
  ));
  return screen.findByRole("button");
}

describe("DailyClaimBadge", () => {
  beforeEach(() => queryClient.clear());

  it("names the game and the action on the claim button", async () => {
    const button = await mountUnclaimed();

    expect(button).toHaveTextContent("Claim daily reward for Genshin Impact");
  });

  it("keeps the visible claim state inside the accessible name", async () => {
    const button = await mountUnclaimed();

    expect(button).toHaveTextContent("Unclaimed");
  });
});
