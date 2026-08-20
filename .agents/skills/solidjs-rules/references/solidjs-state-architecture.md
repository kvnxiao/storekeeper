---
paths: **/*.{ts,tsx,js,jsx}
description: "Architecture for separating business logic from views; state modules own business state, exported accessors and actions, private setters, and components keep only UI-local state."
---

# State Architecture

Keep small, view-specific workflows local. Extract a state module when logic is shared, persistent across unmounts, complex, reusable, or valuable to test without rendering.

## State Modules Own Business Logic (Default)

When an extraction trigger applies, default to one domain per module. The module owns its store, derivations, and mutations, and exports read accessors plus named actions.

```ts
import { createMemo, createRoot } from "solid-js";
import { createStore, produce } from "solid-js/store";

interface CartState {
  items: CartItem[];
}

function createCart() {
  const [state, setState] = createStore<CartState>({ items: [] });

  const itemCount = createMemo(() => state.items.length);
  const total = createMemo(() =>
    state.items.reduce((sum, item) => sum + item.price * item.quantity, 0),
  );

  function addItem(item: CartItem) {
    setState(
      produce((s) => {
        const existing = s.items.find((i) => i.id === item.id);
        if (existing) existing.quantity += 1;
        else s.items.push({ ...item, quantity: 1 });
      }),
    );
  }

  async function applyCoupon(code: string) {
    const discount = await api.validateCoupon(code);
    setState("items", {}, "price", (price) => price * discount);
  }

  async function checkout(details?: CheckoutDetails) {
    const order = await api.checkout(state.items, details);
    setState("items", []);
    return order;
  }

  return { items: () => state.items, itemCount, total, addItem, applyCoupon, checkout };
}

export const cart = createRoot(createCart);
```

The exported root above is a client-lifetime singleton. Retain the disposer for tests, widgets, and roots that can be replaced. In SSR, instantiate the factory once per request instead of sharing a module singleton.

## Never Export the Setter (Required)

Once a state module owns an invariant, its exported actions must be the entire write API. Do not export `setState` or a signal setter; expose named domain actions that preserve the invariant.

```ts
export const [cartState, setCartState] = createStore<CartState>({ items: [] });
```

The module exports its constrained API instead:

```ts
export const cart = createRoot(createCart);
```

## Components Render State (Default)

When business logic has moved into a state module, default components to reading accessors, calling actions, and rendering. Keep a small workflow in the component when it remains specific to that view.

```tsx
const Cart: Component = () => (
  <section>
    <For each={cart.items()}>{(item) => <CartRow item={item} />}</For>
    <output>{cart.total()}</output>
    <button onClick={() => cart.checkout()}>Checkout</button>
  </section>
);
```

## Keep view-specific state local (Default)

Default `createSignal` inside a component for open state, focus, input drafts, animation state, and short workflows used only by that view. Extract the value when it must survive unmount or navigation, another component needs it, or a domain invariant constrains it.

```tsx
const CouponField: Component = () => {
  const [draft, setDraft] = createSignal("");
  return (
    <form onSubmit={() => cart.applyCoupon(draft())}>
      <input value={draft()} onInput={(e) => setDraft(e.currentTarget.value)} />
    </form>
  );
};
```

## Derive in the Module, Not the View (Default)

Default business derivations such as totals, filters, and validity to exported module memos or accessors. Keep presentation-only formatting in the view.

## Server State Belongs to the Query Cache (Default)

When caching, invalidation, retries, deduplication, or preloading matters, default server reads and writes to TanStack Query. Domain modules can export `queryOptions` and `mutationOptions` factories while components consume them with `useQuery` and `useMutation`. Keep the cache as the source of truth unless the application deliberately creates an editable draft, offline snapshot, serialization payload, or cache-independent state.

## When Context Enters (Default)

Context is not a state manager under this architecture; it is an instancing and injection mechanism. Modules answer "the only instance"; context answers "which instance". A module singleton stays the default for app-wide business state in client-only apps. Keep the same factory and deliver it through a provider with a throwing accessor hook (see the stores and state rules) when:

- **SSR**: a module singleton is shared across concurrent requests on the server, leaking one user's state into another's. Instantiate the factory once per request in a root provider; components are unaffected because they only ever see the returned API shape.
- **Per-subtree instances**: state that is one-per-region rather than one-per-app — each wizard's progress, each editor pane, each data grid's sort and filter state. A provider per subtree gives every instance its own factory result where a module singleton would force sharing.
- **Dependency injection**: swapping the implementation a subtree sees — an API client carrying session auth, feature flags, or a stub state module in tests and stories — without module-mocking machinery.
- **Compound components**: parent/child families (Tabs/Tab, Accordion/Item) sharing private coordination state. This UI state is scoped to the family and belongs in the component family rather than a state module.

## Test Modules Without Rendering

State modules let tests call `createCart()` inside `createRoot`, drive actions, and assert on accessors without rendering a component. Component tests can then cover wiring and presentation.
