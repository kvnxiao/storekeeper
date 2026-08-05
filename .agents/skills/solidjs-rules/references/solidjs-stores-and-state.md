---
paths: **/*.{ts,tsx,js,jsx}
description: "SolidJS state rules; signals vs createStore, path setters, produce, reconcile for server snapshots, context patterns, and global state ownership."
---

# Stores and State

## Signals for Values, Stores for Structures

Use `createSignal` for independent single values and `createStore` for nested objects and arrays. Store property access is tracked per property, so updating one field notifies only readers of that field. One giant signal holding a large object updates every reader on any change; that destroys fine-grained reactivity.

## Never Destructure a Store

Stores are proxies; destructuring reads the value once and severs reactivity, exactly like props. Access properties at the point of use inside tracking scopes.

```tsx
// Bad
const { name } = store.user;

// Good
<div>{store.user.name}</div>
```

## Write Through the Setter

Never mutate the store object directly. Use path syntax for targeted updates and `produce` for multi-field or array-mutation updates.

```tsx
const [state, setState] = createStore({ users: [], count: 0 });

// Path syntax
setState("users", 0, "loggedIn", true);
setState("users", (users) => [...users, newUser]);

// produce: localized mutation for compound updates
setState(produce((s) => {
  s.count += 1;
  s.users.push(newUser);
}));
```

## Reconcile Wholesale Replacements

Server state normally lives in the TanStack Query cache, not in stores (see the data fetching rules). When external data does land in a store wholesale — a websocket snapshot, a polled payload outside the cache — wrap it in `reconcile` so unchanged parts keep identity and only real changes propagate. Items match by `id` by default; pass `key` for a different field.

```tsx
setState("todos", reconcile(fetchedTodos));
```

## Context: Provider-Created State, Throwing Accessor

Use context for per-subtree instances and SSR-safe injection; the state architecture rules govern when to reach for it over a module singleton. Create the signals or store inside the provider component, and expose a hook that throws when the provider is missing. A `createContext` default silently masks a missing provider.

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

## Global State Needs a Root

Computations created at module scope leak and warn without an owner; wrap module-level state in `createRoot`. Module shape — exported accessors and named actions with private setters — and the choice between module singletons and context providers are governed by the state architecture rules.
