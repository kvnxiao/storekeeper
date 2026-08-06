---
paths: **/*.{ts,tsx,js,jsx}
description: "Core SolidJS reactivity rules; tracking scopes, derived functions vs createMemo, effects for side effects only, on(), untrack/batch, and reactive ownership."
---

# Reactivity

## Read Signals Inside Tracking Scopes

Dependency tracking happens only when a signal getter is called inside a tracking scope: JSX expressions, `createEffect`, `createMemo`, `createRenderEffect`, or control-flow component props. A read outside those scopes evaluates once and never updates. This is the most common Solid bug; the symptom is "renders once, then never changes".

```tsx
// Bad: computed once at setup, frozen forever
const doubled = count() * 2;

// Good: a derived function re-evaluates wherever it is read in a tracking scope
const doubled = () => count() * 2;
```

## Derived Functions First, `createMemo` When Shared or Expensive

A plain derived function is the default idiom. Reach for `createMemo` when the computation is expensive, when the value is read from multiple reactive contexts (each read of a plain function recomputes), or when you want its equality check to stop downstream updates for unchanged results.

```tsx
// Default: cheap derivation, read in one place
const fullName = () => `${firstName()} ${lastName()}`;

// Memo: expensive or read in several tracking scopes
const sorted = createMemo(() => [...items()].sort(byRank));
```

Do not wrap a simple passthrough in a memo; `createMemo(() => props.id)` adds a graph node for nothing when `() => props.id` suffices.

## Effects Synchronize With the Outside World

`createEffect` is for DOM measurement, subscriptions, logging, and third-party libraries. Never use it to derive state from other state; derive instead.

```tsx
// Bad: state synced through an effect
createEffect(() => setFullName(`${firstName()} ${lastName()}`));

// Good
const fullName = () => `${firstName()} ${lastName()}`;
```

## No Dependency Arrays

Effects and memos track automatically; there is nothing to declare. For explicit dependencies or to skip the first run, use `on`.

```tsx
// Bad: React habit; the second argument is not a dependency array
createEffect(() => console.log(a()), [a]);

// Good: track only `a`; `b()` is read untracked; skip the initial run
createEffect(on(a, (v) => console.log(v, b()), { defer: true }));
```

## `await` Breaks Tracking

Only signal reads before the first `await` are tracked; reads after it are silently untracked, so the effect stops re-running. Never fetch data in an async effect; use TanStack Query or `createResource` (see the data fetching rules).

```tsx
// Bad: `filter()` is read after await and is not tracked
createEffect(async () => {
  const res = await fetch(url());
  applyFilter(await res.json(), filter());
});
```

## `untrack` and `batch`

Use `untrack(() => sig())` to read a value inside a tracking scope without subscribing, such as comparing against a previous value. Use `batch` to group multiple writes so dependents run once; event handlers and store setters already batch, so it mainly matters in async callbacks and imperative code.

```tsx
batch(() => {
  setItems(next);
  setSelected(undefined);
});
```

## Own Every Computation

Computations created outside any root leak and warn. Components provide owners automatically; for reactive graphs outside the component tree (module-level stores, imperative widgets), wrap creation in `createRoot` and keep the `dispose` handle. In async continuations (`setTimeout`, promise callbacks) the owner is lost; capture it with `getOwner()` and restore with `runWithOwner` when creating computations there.
