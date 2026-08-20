---
paths: **/*.{test,spec}.{ts,tsx}
description: "SolidJS testing rules; vitest + @solidjs/testing-library stack, render takes a thunk, role-based queries, renderHook for primitives, and testEffect for reactive assertions."
---

# Testing

## Recommended Stack (Default)

Default to `vitest`, `jsdom`, `@solidjs/testing-library`, `@testing-library/user-event`, and `@testing-library/jest-dom`. Use an existing compatible test stack when migration cost outweighs consistency. `vite-plugin-solid` configures Vitest for Solid; if `solid-js` loads twice, correct `resolve.conditions` or dependency inlining.

## `render` Takes a Function (Required)

The thunk preserves Solid's ownership and reactive root. Cleanup is automatic per test.

```tsx
render(<Counter />);
```

Pass a render function to create the ownership root:

```tsx
const { getByRole } = render(() => <Counter />);
```

## Test Behavior, Not Implementation (Default)

Default DOM queries to roles and accessible names, and drive interaction through `user-event`. Use a test ID when the element has no accessible semantic or stable visible text. Assert on behavior instead of signal internals.

```tsx
const user = userEvent.setup();
render(() => <Counter />);

const button = screen.getByRole("button", { name: /increment/i });
await user.click(button);
expect(screen.getByText("Count: 1")).toBeInTheDocument();
```

## Providers via `wrapper`, Routes via a Memory Router (Default)

```tsx
render(() => <Profile />, {
  wrapper: (props) => <AuthProvider>{props.children}</AuthProvider>,
});

import { RouterProvider, createMemoryHistory, createRouter } from "@tanstack/solid-router";

const router = createRouter({
  routeTree,
  history: createMemoryHistory({ initialEntries: ["/article/12345"] }),
});

render(() => <RouterProvider router={router} />);
await screen.findByRole("heading");
```

The testing library's `location` render option is `@solidjs/router`-only and does not apply to this stack.

## `renderHook` for Primitives (Default)

Default custom primitives to `renderHook` when they do not need host DOM or provider behavior.

```tsx
const { result, cleanup } = renderHook(useCounter, { initialProps: [5] });
expect(result.count()).toBe(5);
```

## `testEffect` for Reactive Assertions (Default)

When an assertion depends on a scheduled reactive update, use `testEffect` to run it inside an effect and resolve through `done()`.

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
