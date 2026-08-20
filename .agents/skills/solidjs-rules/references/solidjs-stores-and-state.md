---
paths: **/*.{ts,tsx,js,jsx}
description: "SolidJS state rules; signals vs createStore, path setters, produce, reconcile for server snapshots, unwrap at proxy-hostile boundaries, context patterns, and global state ownership."
---

# Stores and State

## Signals for Values, Stores for Structures (Default)

Default independent scalar values to `createSignal` and nested objects or arrays to `createStore`. Keep a structured value in one signal when consumers intentionally update as one unit.

## Never Destructure a Store (Required)

Stores are proxies; destructuring reads the value once and severs reactivity, exactly like props. Access properties at the point of use inside tracking scopes.

```tsx
const { name } = store.user;
```

Property access inside a tracking scope remains reactive:

```tsx
<div>{store.user.name}</div>
```

## Write Through the Setter (Required)

Never mutate the store object directly. Use path syntax for targeted updates and `produce` for multi-field or array-mutation updates.

```tsx
const [state, setState] = createStore({ users: [], count: 0 });

setState("users", 0, "loggedIn", true);
setState("users", (users) => [...users, newUser]);

setState(produce((s) => {
  s.count += 1;
  s.users.push(newUser);
}));
```

## Reconcile Wholesale Replacements (Conditional)

Server state normally lives in the TanStack Query cache, not in stores (see the data fetching rules). When external data does land in a store wholesale — a websocket snapshot, a polled payload outside the cache — wrap it in `reconcile` so unchanged parts keep identity and only real changes propagate. Items match by `id` by default; pass `key` for a different field.

```tsx
setState("todos", reconcile(fetchedTodos));
```

## Unwrap Before Leaving Reactivity (Required)

Store values are proxies at every nested level. Library values backed by a store are proxies too, including solid-query's `query.data` (see the data fetching rules). Call `unwrap` whenever store data exits the reactive system: structured cloning, `postMessage`/Workers, IndexedDB, history state, IPC, or a snapshot kept for later comparison. Structured-clone surfaces throw `DataCloneError` on a proxy; APIs that instead walk the object (`JSON.stringify`, permissive deep-clone utilities) succeed but read every property through the traps, subscribing the current tracking scope to the entire tree. `unwrap` is a no-op on plain objects, so apply it defensively at these boundaries.

```ts
form.initialize(structuredClone(query.data));
```

Unwrapping produces plain data for the clone boundary:

```ts
form.initialize(structuredClone(unwrap(query.data)));
```

## Context: Provider-Created State, Throwing Accessor (Default)

For per-subtree instances and SSR-safe injection, default state creation to the provider component and expose a hook that throws when the provider is missing. Use a context default only when a real fallback instance is part of the contract.

```tsx
interface CounterValue {
  count: Accessor<number>;
  increment: () => void;
}

const CounterContext = createContext<CounterValue>();

export const CounterProvider: ParentComponent = (props) => {
  const [count, setCount] = createSignal(0);
  const value: CounterValue = {
    count,
    increment: () => setCount((c) => c + 1),
  };
  return (
    <CounterContext.Provider value={value}>
      {props.children}
    </CounterContext.Provider>
  );
};

export function useCounter() {
  const ctx = useContext(CounterContext);
  if (!ctx) throw new Error("useCounter must be used within CounterProvider");
  return ctx;
}
```

The provider exposes named verbs, not the raw setter — the same write-API discipline as state modules.

## Global State Needs a Root (Required)

Computations created at module scope leak and emit warnings without an owner; wrap module-level state in `createRoot`. Module shape — exported accessors and named actions with private setters — and the choice between module singletons and context providers are governed by the state architecture rules.
