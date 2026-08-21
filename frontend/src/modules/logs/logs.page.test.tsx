import { render, screen } from "@solidjs/testing-library";
import { QueryClientProvider } from "@tanstack/solid-query";
import { userEvent } from "@testing-library/user-event";
import { afterAll, beforeAll, describe, expect, it, vi } from "vite-plus/test";
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

// The virtualizer sizes its range from the container's offsetHeight, which
// jsdom reports as 0 for every element; without a height it renders no rows.
const offsetHeight = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "offsetHeight");

beforeAll(() => {
  Object.defineProperty(HTMLElement.prototype, "offsetHeight", {
    configurable: true,
    get: () => 600,
  });
});

afterAll(() => {
  if (offsetHeight) {
    Object.defineProperty(HTMLElement.prototype, "offsetHeight", offsetHeight);
  }
});

function renderLogsPage(): void {
  render(() => (
    <QueryClientProvider client={queryClient}>
      <LogsPage />
    </QueryClientProvider>
  ));
}

describe("LogsPage", () => {
  it("renders a row per entry at or above the minimum level", async () => {
    renderLogsPage();

    expect(await screen.findByText("Storekeeper starting")).toBeInTheDocument();
    expect(screen.getByText("Kuro API requested retry")).toBeInTheDocument();
    expect(screen.getByText("2 shown")).toBeInTheDocument();
  });

  it("holds back an entry below the minimum level", async () => {
    renderLogsPage();
    await screen.findByText("Storekeeper starting");

    expect(
      screen.queryByText("Sign endpoint reports today already signed"),
    ).not.toBeInTheDocument();
  });

  it("narrows the list to the entries matching the filter", async () => {
    renderLogsPage();
    await screen.findByText("Storekeeper starting");

    await userEvent.type(screen.getByLabelText("Filter"), "Kuro");

    expect(screen.getByText("Kuro API requested retry")).toBeInTheDocument();
    expect(screen.queryByText("Storekeeper starting")).not.toBeInTheDocument();
  });
});
