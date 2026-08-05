---
paths: **/*.{test,spec}.{ts,tsx}
description: "SolidJS testing rules; vitest + @solidjs/testing-library stack, render takes a thunk, role-based queries, renderHook for primitives, and testEffect for reactive assertions."
---

# Testing

## Blessed Stack

Use `vitest` + `jsdom` + `@solidjs/testing-library` + `@testing-library/user-event` + `@testing-library/jest-dom`. `vite-plugin-solid` ≥ 2.8.2 configures vitest for Solid automatically; if `solid-js` loads twice (symptoms: "dispose is undefined", router failing to load), fix `resolve.conditions`/dep inlining rather than working around it in tests.

## `render` Takes a Function

The thunk preserves Solid's ownership and reactive root. Cleanup is automatic per test.

```tsx
// Bad
render(<Counter />);

// Good
const { getByRole } = render(() => <Counter />);
```

## Test Behavior, Not Implementation

Query the DOM the way assistive technology does: by role and accessible name, not test IDs or CSS classes. Drive interaction through `user-event`. Do not assert on signal internals.

```tsx
const user = userEvent.setup();
render(() => <Counter />);

const button = screen.getByRole("button", { name: /increment/i });
await user.click(button);
expect(screen.getByText("Count: 1")).toBeInTheDocument();
```

## Providers via `wrapper`, Routes via a Memory Router

```tsx
render(() => <Profile />, {
  wrapper: (props) => <AuthProvider>{props.children}</AuthProvider>,
});

// TanStack Router: render a real router over memory history; routing is
// async, so the first query must be an async findBy*
import { RouterProvider, createMemoryHistory, createRouter } from "@tanstack/solid-router";

const router = createRouter({
  routeTree,
  history: createMemoryHistory({ initialEntries: ["/article/12345"] }),
});

render(() => <RouterProvider router={router} />);
await screen.findByRole("heading");
```

The testing library's `location` render option is `@solidjs/router`-only and does not apply to this stack.

## `renderHook` for Primitives

Test custom primitives without a host component.

```tsx
const { result, cleanup } = renderHook(useCounter, { initialProps: [5] });
expect(result.count()).toBe(5);
```

## `testEffect` for Reactive Assertions

Asserting "signal change causes X" races the scheduler if done inline. `testEffect` runs assertions inside an effect and resolves when `done()` is called.

```tsx
await testEffect((done) =>
  createEffect((run: number = 0) => {
    if (run === 0) setCount(1);
    else {
      expect(count()).toBe(1);
      done();
    }
    return run + 1;
  }),
);
```
