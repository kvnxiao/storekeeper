---
paths: **/*.{ts,tsx,js,jsx}
description: "Core SolidJS reactivity rules; tracking scopes, derived functions vs createMemo, effects for side effects only, on(), untrack/batch, and reactive ownership."
---

# Reactivity

## Read Signals Inside Tracking Scopes (Required)

Dependency tracking happens only when a signal getter is called inside a tracking scope: JSX expressions, `createEffect`, `createMemo`, `createRenderEffect`, or control-flow component props. A read outside those scopes evaluates once and never updates. A common symptom is "renders once, then never changes". A setup-time read is fixed, while an accessor defers evaluation to a tracking scope:

```tsx
const frozenDoubled = count() * 2;

const doubled = () => count() * 2;
```

## Derived Functions First, `createMemo` When Shared or Expensive (Default)

A plain derived function is the default idiom. Use `createMemo` when the computation is expensive, when the value is read from multiple reactive contexts (each read of a plain function recomputes), or when you want its equality check to stop downstream updates for unchanged results.

```tsx
const fullName = () => `${firstName()} ${lastName()}`;

const sorted = createMemo(() => [...items()].sort(byRank));
```

Avoid wrapping a simple passthrough in a memo unless downstream equality suppression matters; `() => props.id` is otherwise sufficient.

## Effects Synchronize With the Outside World (Default)

Default to effects for DOM measurement, subscriptions, logging, and third-party libraries. Derive state with a function or memo unless an external system must receive the change. An effect that writes a signal it also tracks loops.

```tsx
createEffect(() => setFullName(`${firstName()} ${lastName()}`));
```

The direct derivation tracks its inputs without a write-back effect:

```tsx
const fullName = () => `${firstName()} ${lastName()}`;
```

## No Dependency Arrays (Required)

Effects and memos track automatically; there is nothing to declare. For explicit dependencies or to skip the first run, use `on`.

The second argument is not a dependency-array API:

```tsx
createEffect(() => console.log(a()), [a]);
```

`on` defines explicit dependencies and can defer the initial run:

```tsx
createEffect(on(a, (v) => console.log(v, b()), { defer: true }));
```

## `await` Breaks Tracking (Required)

Only signal reads before the first `await` are tracked; reads after it are silently untracked, so the effect stops re-running. Never fetch data in an async effect; use TanStack Query or `createResource` (see the data fetching rules).

```tsx
createEffect(async () => {
  const res = await fetch(url());
  applyFilter(await res.json(), filter());
});
```

## `untrack` and `batch` (Default)

Default to `untrack(() => sig())` when a tracking scope must read a value without subscribing, such as a comparison with a previous value. Default to `batch` for grouped writes in async callbacks and imperative code; event handlers and store setters already batch.

```tsx
batch(() => {
  setItems(next);
  setSelected(undefined);
});
```

## Own Every Computation (Required)

Computations created outside any root leak and emit warnings. Components provide owners automatically; for reactive graphs outside the component tree (module-level stores, imperative widgets), wrap creation in `createRoot` and keep the `dispose` handle. In async continuations (`setTimeout`, promise callbacks) the owner is lost; capture it with `getOwner()` and restore it with `runWithOwner` when creating computations in the continuation.
