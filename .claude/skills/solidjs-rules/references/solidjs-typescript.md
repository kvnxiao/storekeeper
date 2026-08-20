---
paths: **/*.{ts,tsx}
description: "TypeScript rules for SolidJS; jsx preserve config, Component/ParentComponent/VoidComponent, Accessor/Setter types, typed events, and ref typing."
---

# TypeScript

## Compiler Configuration (Required)

Solid TypeScript projects must use `.tsx` files with `"jsx": "preserve"` and `"jsxImportSource": "solid-js"` in `tsconfig.json`. Default `strict` mode to on; relax an individual strictness option only for a documented migration or compatibility constraint.

## Component Types Do Not Imply Children (Default)

`Component<P>` is `(props: P) => JSX.Element` with no implicit children. Default to the component type that states the children contract; use a function signature when generic props require it.

```tsx
const Card: ParentComponent<CardProps> = (props) => (
  <div class="card">{props.children}</div>
);

const Icon: VoidComponent<IconProps> = () => <svg />;
```

- `ParentComponent<P>` / `ParentProps<P>` add `children?: JSX.Element`.
- `VoidComponent<P>` forbids children.
- `FlowComponent<P, C>` requires children; set `C` to a function type for control-flow-style render callbacks.
- Return type is `JSX.Element`, which already covers elements, arrays, strings, numbers, and null.

## Pass Accessors, Not Values (Required)

A signature taking `T` freezes the value at call time. Take `Accessor<T>` (or a prop) only when the callee must observe later values; take `T` when the call is a snapshot.

```tsx
function useTitle(title: string) {}
```

An accessor preserves the reactive read:

```tsx
function useTitle(title: Accessor<string>) {
  createEffect(() => (document.title = title()));
}
```

`createSignal<T>()` without an initial value yields `Accessor<T | undefined>`; providing an initial value narrows to `T`. A stored signal pair is `Signal<T> = [Accessor<T>, Setter<T>]`.

## Function-Valued Setters (Required)

A function passed to a setter is treated as an updater. To store a function in a signal, wrap it.

```tsx
setHandler(() => onSelect);
```

## Typed Events (Default)

`JSX.EventHandler<HTMLInputElement, InputEvent>` types `event.currentTarget` to the element; `event.target` stays the generic `Element`. Inline handlers get inference automatically.

```tsx
const onInput: JSX.EventHandler<HTMLInputElement, InputEvent> = (e) =>
  setText(e.currentTarget.value);
```

## Ref Typing (Default)

Default unconditional refs to definite assignment and conditional refs to an optional type with guards or a signal ref.

```tsx
let el!: HTMLDivElement;
let maybe: HTMLDivElement | undefined;
```

## Store Updates (Default)

Default stores to an explicit `createStore<StateShape>({ … })` type. For deep updates, prefer `produce` when path-setter types obscure the intended mutation.
