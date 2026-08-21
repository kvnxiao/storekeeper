import { render, screen } from "@solidjs/testing-library";
import { QueryClientProvider } from "@tanstack/solid-query";
import { userEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vite-plus/test";
import { queryClient } from "@/modules/core/core.queryClient";
import { LogsPage } from "@/modules/logs/components/LogsPage";

function line(level: string, message: string): string {
  return JSON.stringify({
    timestamp: "2026-08-21T04:15:09.123456Z",
    level,
    fields: { message },
    target: "storekeeper_app_tauri::scheduled_claim",
  });
}

const TAIL = [
  line("INFO", "Storekeeper starting"),
  line("WARN", "Kuro API requested retry"),
  line("DEBUG", "Sign endpoint reports today already signed"),
  "{ truncated",
];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (command: string) => (command === "read_log_tail" ? TAIL : undefined)),
}));

// jsdom reports every element as zero-height, so the virtualizer renders an
// empty range and the assertions below read the count rather than the rows.
function renderLogsPage(): void {
  render(() => (
    <QueryClientProvider client={queryClient}>
      <LogsPage />
    </QueryClientProvider>
  ));
}

describe("LogsPage", () => {
  it("counts the entries at or above the minimum level", async () => {
    renderLogsPage();

    expect(await screen.findByText("2 shown")).toBeInTheDocument();
    expect(screen.queryByText("Reading logs...")).not.toBeInTheDocument();
  });

  it("narrows the count to the entries matching the filter", async () => {
    renderLogsPage();
    await screen.findByText("2 shown");

    await userEvent.type(screen.getByLabelText("Filter"), "Kuro");

    expect(await screen.findByText("1 shown")).toBeInTheDocument();
  });
});
