import { render, screen } from "@solidjs/testing-library";
import { QueryClientProvider } from "@tanstack/solid-query";
import { beforeEach, describe, expect, it, vi } from "vite-plus/test";
import { queryClient } from "@/modules/core/core.queryClient";
import { DashboardContent } from "@/modules/dashboard/components/DashboardContent";

const CONFIG_ERROR = "config file is unreadable";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (command: string) => {
    switch (command) {
      case "get_config":
        throw new Error(CONFIG_ERROR);
      case "get_all_resources":
        return { games: {}, lastUpdated: Temporal.Instant.fromEpochMilliseconds(0).toString() };
      default:
        return {};
    }
  }),
}));

describe("DashboardContent", () => {
  beforeEach(() => queryClient.clear());

  it("surfaces a config load failure instead of the empty state", async () => {
    const { container } = render(() => (
      <QueryClientProvider client={queryClient}>
        <DashboardContent />
      </QueryClientProvider>
    ));

    await screen.findByText(new RegExp(CONFIG_ERROR));
    expect(container).not.toHaveTextContent("No games configured");
  });
});
