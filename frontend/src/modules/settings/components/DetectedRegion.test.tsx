import { QueryClientProvider } from "@tanstack/solid-query";
import { render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vite-plus/test";
import { queryClient } from "@/modules/core/core.queryClient";
import { GameId } from "@/modules/games/games.types";
import { DetectedRegion } from "@/modules/settings/components/DetectedRegion";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

function mount(gameId: GameId, uid: string) {
  return render(() => (
    <QueryClientProvider client={queryClient}>
      <DetectedRegion gameId={gameId} uid={uid} />
    </QueryClientProvider>
  ));
}

describe("DetectedRegion", () => {
  beforeEach(() => {
    queryClient.clear();
    invoke.mockReset();
  });

  it("shows the region the backend read from the UID", async () => {
    invoke.mockResolvedValue("china_hmt");

    mount(GameId.GenshinImpact, "900000001");

    expect(await screen.findByText(/china_hmt/)).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("detect_region", {
      gameId: GameId.GenshinImpact,
      uid: "900000001",
    });
  });

  it("warns when the backend cannot place the UID on a server", async () => {
    invoke.mockRejectedValue(new Error("Could not determine region from UID"));

    mount(GameId.GenshinImpact, "400000001");

    expect(await screen.findByText(/could not determine/i)).toBeInTheDocument();
  });

  it("asks nothing while the UID field is empty", () => {
    mount(GameId.GenshinImpact, "");

    expect(invoke).not.toHaveBeenCalled();
  });
});
