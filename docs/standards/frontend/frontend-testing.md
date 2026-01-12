# Testing Standards

## Overview

Testing uses [Vitest](https://vitest.dev/) with [Testing Library](https://testing-library.com/) for React component testing.

## Test File Organization

```
src/
├── components/
│   ├── ui/
│   │   ├── Button.tsx
│   │   └── Button.test.tsx    # Co-located test
│   └── __tests__/             # Alternative: grouped tests
│       └── GameSection.test.tsx
└── utils/
    ├── resourceFormatting.ts
    └── resourceFormatting.test.ts
```

Name test files `*.test.ts` or `*.test.tsx`.

## Component Testing

### Query by Accessibility

Use accessible queries over implementation details:

```tsx
// Bad: Implementation detail
const button = container.querySelector(".btn-primary");

// Bad: Test ID (last resort)
const button = screen.getByTestId("submit-button");

// Good: Accessible query
const button = screen.getByRole("button", { name: /submit/i });
const input = screen.getByLabelText("Email");
const heading = screen.getByRole("heading", { level: 1 });
```

### Test Visible Behavior

```tsx
// Bad: Testing internal state
expect(component.state.isOpen).toBe(true);

// Good: Testing visible behavior
expect(screen.getByRole("dialog")).toBeVisible();
```

## Provider Wrapper

Wrap renders with necessary providers:

```tsx
// test-utils.tsx
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Provider as JotaiProvider } from "jotai";

function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
}

export function renderWithProviders(ui: React.ReactElement) {
  const queryClient = createTestQueryClient();
  return render(
    <QueryClientProvider client={queryClient}>
      <JotaiProvider>{ui}</JotaiProvider>
    </QueryClientProvider>
  );
}
```

## Testing Async Operations

### Mocking Tauri

```tsx
// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

it("displays resources after fetch", async () => {
  vi.mocked(invoke).mockResolvedValue(mockResources);

  renderWithProviders(<HomePage />);

  await waitFor(() => {
    expect(screen.getByText("Genshin Impact")).toBeInTheDocument();
  });
});
```

### Testing Loading States

```tsx
it("shows loading state initially", () => {
  vi.mocked(invoke).mockImplementation(() => new Promise(() => {})); // Never resolves

  renderWithProviders(<HomePage />);

  expect(screen.getByRole("progressbar")).toBeInTheDocument();
});
```

## Testing Jotai Atoms

```tsx
import { useAtomValue } from "jotai";

it("updates tick every minute", async () => {
  vi.useFakeTimers();

  const TestComponent = () => {
    const tick = useAtomValue(tickAtom);
    return <span data-testid="tick">{tick}</span>;
  };

  renderWithProviders(<TestComponent />);
  const initial = screen.getByTestId("tick").textContent;

  await vi.advanceTimersByTimeAsync(60_000);

  expect(screen.getByTestId("tick").textContent).not.toBe(initial);

  vi.useRealTimers();
});
```

## Test Structure

Follow AAA pattern (Arrange, Act, Assert):

```tsx
describe("StaminaCard", () => {
  it("displays time remaining until full", () => {
    // Arrange
    const data: StaminaResource = {
      current: 100,
      max: 200,
      fullAt: new Date(Date.now() + 3600000).toISOString(),
      regenRateSeconds: 360,
    };

    // Act
    renderWithProviders(<StaminaCard type="stamina" data={data} />);

    // Assert
    expect(screen.getByText(/1 hour/i)).toBeInTheDocument();
  });
});
```

## Checklist

- [ ] Tests query by accessible roles/labels
- [ ] Test visible behavior, not implementation
- [ ] Async operations use `waitFor`
- [ ] Tauri APIs properly mocked
- [ ] Providers wrapped via test utility
- [ ] Fake timers cleaned up after tests
