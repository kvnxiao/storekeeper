---
paths: **/*.{tsx,jsx}
description: "SolidJS component rules; components run once, never destructure props, mergeProps/splitProps, the children helper, and Dynamic components."
---

# Components and Props

## Components Run Exactly Once (Required)

A component body is setup code, not a render function. The body runs once, so locals computed there are computed once and closures remain stable for the component's lifetime. Put per-update logic in JSX expressions, derived functions, memos, or effects.

## No Early Returns on Reactive State (Required)

An `if`/`return` in the body evaluates once and freezes the decision forever. Put conditionals in JSX with `<Show>` or `<Switch>`.

```tsx
interface ProfileProps {
  loading: boolean;
  user: User;
}

const Profile: Component<ProfileProps> = (props) => {
  if (props.loading) return <Spinner />;
  return <div>{props.user.name}</div>;
};
```

The control-flow component tracks `props.loading`:

```tsx
const Profile: Component<ProfileProps> = (props) => (
  <Show when={!props.loading} fallback={<Spinner />}>
    <div>{props.user.name}</div>
  </Show>
);
```

## Never Destructure Props (Required)

Props are getter-backed objects; destructuring or copying to a local evaluates the getter once and severs reactivity. Access `props.x` at the point of use, or re-wrap as an accessor.

```tsx
const { name } = props;
const name = props.name;
```

```tsx
const view = () => <div>{props.name}</div>;
const name = () => props.name;
```

## Defaults With `mergeProps`, Splitting With `splitProps` (Required)

Default-parameter destructuring breaks reactivity; rest-spread destructuring does too. Use the helpers, which preserve getters.

```tsx
const Button = ({ size = "md", ...rest }) => {};
```

The Solid helpers preserve getter-backed properties:

```tsx
interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  size?: "sm" | "md" | "lg";
}

const Button: ParentComponent<ButtonProps> = (props) => {
  const merged = mergeProps({ size: "md" }, props);
  const [local, rest] = splitProps(merged, ["size", "children"]);
  return <button data-size={local.size} {...rest}>{local.children}</button>;
};
```

## Resolve Children With the `children` Helper (Required)

`props.children` is a getter that may create DOM on each access; reading it twice mounts it twice. If children are read more than once, or inspected or iterated, resolve them once with the `children` helper and use `resolved()` or `resolved.toArray()`.

```tsx
import { children } from "solid-js";

const List: ParentComponent = (props) => {
  const resolved = children(() => props.children);
  return <ul>{resolved.toArray().map((child) => <li>{child}</li>)}</ul>;
};
```

## Switch Components With `<Dynamic>` (Default)

Default a tag or component chosen by a signal to `<Dynamic component={...}>` from `solid-js/web`; use static JSX when the component identity cannot change.

```tsx
<Dynamic component={views[mode()]} item={props.item} />
```

## Primitives Do Not Follow React Hook Rules (Default)

Signals, memos, and effects may be created in conditionals, loops, event handlers, or outside components when an owner exists. React hook-driven restructuring is unnecessary.
