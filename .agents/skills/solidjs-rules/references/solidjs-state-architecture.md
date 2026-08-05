---
paths: **/*.{ts,tsx,js,jsx}
description: "House architecture for separating business logic from views; state modules own business state, exported accessors and actions, private setters, and components keep only UI-local state."
---

# State Architecture

Business state and the logic that mutates it live in dedicated state modules, not in components. Components are thin views: they read exported accessors, call exported actions, and hold only UI-local state. Solid makes this native — signals, stores, and memos work at module scope under `createRoot` — so no third-party state library is needed to get Jotai-style separation.

## State Modules Own Business Logic

One domain per module (for example `src/state/cart.ts`, or the feature folder's `state.ts`). The module owns the store, derived values, and every mutation, and exports read accessors plus named action functions.

```ts
// src/state/cart.ts
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

## Never Export the Setter

The exported actions are the entire write API. Exporting `setState` (or a signal's setter) lets any component mutate business state arbitrarily, which destroys the invariants the module exists to hold and makes writes impossible to trace. Every mutation should be a named domain verb that is grep-able and testable.

```ts
// Bad: any component can now write anything
export const [cartState, setCartState] = createStore<CartState>({ items: [] });

// Good: writes only happen through domain verbs
export const cart = createRoot(createCart);
```

## Components Stay Dumb

A component that fetches, derives, and mutates business state fuses the view and the viewmodel; it can only be tested by rendering it, and the logic cannot be reused. Components should read accessors, call actions, and render.

```tsx
// Bad: view and business logic fused in the component
const Cart: Component = () => {
  const [items, setItems] = createSignal<CartItem[]>([]);
  const total = () =>
    items().reduce((sum, item) => sum + item.price * item.quantity, 0);
  const applyCoupon = async (code: string) => {
    const discount = await api.validateCoupon(code);
    setItems(items().map((i) => ({ ...i, price: i.price * discount })));
  };
  /* … */
};

// Good: thin view over the cart module
const Cart: Component = () => (
  <section>
    <For each={cart.items()}>{(item) => <CartRow item={item} />}</For>
    <output>{cart.total()}</output>
    <button onClick={() => cart.checkout()}>Checkout</button>
  </section>
);
```

## Local State Is UI State Only

`createSignal` inside a component is for state that exists only for that view: open/collapsed flags, hover and focus, in-progress input drafts, transient animation state. The litmus test: if the value must survive unmount or navigation, if another component needs it, or if a business rule reads or constrains it, it belongs in a state module.

```tsx
const CouponField: Component = () => {
  const [draft, setDraft] = createSignal("");        // UI-local: fine
  return (
    <form onSubmit={() => cart.applyCoupon(draft())}>
      <input value={draft()} onInput={(e) => setDraft(e.currentTarget.value)} />
    </form>
  );
};
```

## Derive in the Module, Not the View

Business derivations (totals, filters, validity) are exported memos or accessors on the module. A component that recomputes them re-encodes business rules in the view, and sibling views drift apart.

## Server State Belongs to the Query Cache

Server reads and writes go through TanStack Query (see the data fetching rules): domain modules export `queryOptions` and `mutationOptions` factories, and components consume them with `useQuery`/`useMutation`. Do not copy query data into stores — the cache is the source of truth for server state, and state modules hold client business state only. Module actions remain the home for client-side workflows and for the domain logic mutations delegate to (validation, optimistic-update shaping, multi-step orchestration like `checkout` above).

## When Context Enters

Context is not a state manager under this architecture; it is an instancing and injection mechanism. Modules answer "the only instance"; context answers "which instance". A module singleton stays the default for app-wide business state in client-only apps. Keep the same factory and deliver it through a provider with a throwing accessor hook (mechanics in the stores and state rules) when:

- **SSR**: a module singleton is shared across concurrent requests on the server, leaking one user's state into another's. Instantiate the factory once per request in a root provider; components are unaffected because they only ever see the returned API shape.
- **Per-subtree instances**: state that is one-per-region rather than one-per-app — each wizard's progress, each editor pane, each data grid's sort and filter state. A provider per subtree gives every instance its own factory result where a module singleton would force sharing.
- **Dependency injection**: swapping the implementation a subtree sees — an API client carrying session auth, feature flags, or a stub state module in tests and stories — without module-mocking machinery.
- **Compound components**: parent/child families (Tabs/Tab, Accordion/Item) sharing private coordination state. That is UI state scoped to the family and inherently multi-instance; it never belongs in a state module.

## Test Modules Without Rendering

This split is what makes business logic testable as plain TypeScript: call `createCart()` inside `createRoot` in a test, drive actions, and assert on accessors — no component render, no DOM. Component tests then only need to cover wiring and presentation.
