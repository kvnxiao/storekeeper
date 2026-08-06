---
paths: **/*.{tsx,jsx}
description: "SolidJS control-flow components; Show over ternaries, For vs Index over .map(), Switch/Match, Portal, and keyed semantics."
---

# Control Flow

Raw `.map()` and ternaries work in Solid, but the whole expression re-evaluates whenever any dependency changes: `.map()` recreates every DOM node in the list, and a ternary recreates its branch on every value change, not just on truthiness change. The control-flow components memoize and diff instead.

## `<Show>` for Conditionals

```tsx
// Bad: branch re-created on every user() change
{user() ? <Profile user={user()} /> : <Login />}

// Good: re-renders only when truthiness flips
<Show when={user()} fallback={<Login />}>
  <Profile user={user()} />
</Show>
```

The callback form narrows nullability and is the idiomatic way to consume the `when` value; non-keyed, it receives an accessor.

```tsx
<Show when={user()}>{(u) => <div>{u().name}</div>}</Show>
```

`keyed` passes the value directly but re-creates the whole branch whenever the `when` value changes identity. Use it only when remount-on-change is wanted; avoid it for frequently-changing references.

## `<For>` for Object Lists, `<Index>` for Value Slots

There is no `key` prop. `<For>` keys by item reference: nodes move and are reused when objects reorder. `<Index>` fixes nodes by position and passes the item as a signal: use it for lists of primitives or fixed slots whose contents change (form inputs), where `<For>` would destroy and recreate nodes and drop input focus.

```tsx
// Objects with stable identity that may reorder
<For each={todos()}>{(todo, i) => <TodoRow todo={todo} index={i()} />}</For>

// Value-typed list in fixed positions; item is the signal
<Index each={inputs()}>{(value, i) => <input value={value()} />}</Index>
```

Rule of thumb: `<For>` is for moving objects; `<Index>` is for changing values in fixed slots.

## `<Switch>`/`<Match>` for Multi-Branch Conditionals

First matching `<Match>` wins. Prefer this over nested ternaries or chained `<Show>`s.

```tsx
<Switch fallback={<NotFound />}>
  <Match when={state.route === "home"}><Home /></Match>
  <Match when={state.route === "settings"}><Settings /></Match>
</Switch>
```

## `<Portal>` for Out-of-Hierarchy Rendering

Use `<Portal>` for modals and tooltips. It mounts to `document.body` by default, keeps component-tree context, and is client-only (skipped in SSR output and hydration).
