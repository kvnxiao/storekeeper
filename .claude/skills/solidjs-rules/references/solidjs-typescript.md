---
paths: **/*.{ts,tsx}
description: "TypeScript rules for SolidJS; jsx preserve config, Component/ParentComponent/VoidComponent, Accessor/Setter types, typed events, and ref typing."
---

# TypeScript

## Compiler Configuration

Use `.tsx` files with `"jsx": "preserve"` and `"jsxImportSource": "solid-js"` in `tsconfig.json`, with `strict` mode on.

## Component Types Do Not Imply Children

`Component<P>` is `(props: P) => JSX.Element` with no implicit children — the `React.FC` habit of free `children` does not carry over. Pick the type that states the contract:

```tsx
const Card: ParentComponent<CardProps> = (props) => (
  <div class="card">{props.children}</div>
);

const Icon: VoidComponent<IconProps> = (props) => <svg>{/* … */}</svg>;
```

- `ParentComponent<P>` / `ParentProps<P>` add `children?: JSX.Element`.
- `VoidComponent<P>` forbids children.
- `FlowComponent<P, C>` requires children; set `C` to a function type for control-flow-style render callbacks.
- Return type is `JSX.Element`, which already covers elements, arrays, strings, numbers, and null.

## Pass Accessors, Not Values

A signature taking `T` freezes the value at call time. Take `Accessor<T>` (or a prop) so the callee reads reactively.

```tsx
// Bad: value captured once
function useTitle(title: string) {}

// Good
function useTitle(title: Accessor<string>) {
  createEffect(() => (document.title = title()));
}
```

`createSignal<T>()` without an initial value yields `Accessor<T | undefined>`; providing an initial value narrows to `T`. A stored signal pair is `Signal<T> = [Accessor<T>, Setter<T>]`.

## Setter Gotcha for Function Values

A function passed to a setter is treated as an updater. To store a function in a signal, wrap it.

```tsx
setHandler(() => onSelect);
```

## Typed Events

`JSX.EventHandler<HTMLInputElement, InputEvent>` types `event.currentTarget` to the element; `event.target` stays the generic `Element`. Inline handlers get inference for free.

```tsx
const onInput: JSX.EventHandler<HTMLInputElement, InputEvent> = (e) =>
  setText(e.currentTarget.value);
```

## Ref Typing

Use definite assignment for unconditional refs, and an optional type with guards (or a signal ref) for conditional ones.

```tsx
let el!: HTMLDivElement;                        // always rendered
let maybe: HTMLDivElement | undefined;          // conditionally rendered
```

## Store Updates

Type stores at creation with `createStore<StateShape>({ … })`. Path setters are typed but get unwieldy on deep paths; `produce` often types more cleanly for deep updates.
