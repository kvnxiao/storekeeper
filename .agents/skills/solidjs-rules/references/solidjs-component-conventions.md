---
paths: **/*.{tsx,jsx}
description: "House conventions for SolidJS component files; one exported component per file, typed Component const declarations from solid-js, and <ComponentName>Props naming."
---

# Component Conventions

House standards for component files. Framework mechanics (props reactivity, children, component types) live in the other rules; these conventions govern how components are declared and organized.

## One Exported Component Per File

Each `.tsx`/`.jsx` file exports at most one component. Unexported helper subcomponents may live in the same file; the moment one is needed elsewhere, move it to its own file instead of exporting a second component.

```tsx
// Bad: two exported components in one file
export const UserCard: Component<UserCardProps> = (props) => { /* … */ };
export const UserAvatar: Component<UserAvatarProps> = (props) => { /* … */ };

// Good: UserAvatar.tsx exports only UserAvatar; helpers stay private
interface InitialsProps {
  initials: string;
}

const Initials: VoidComponent<InitialsProps> = (props) => (
  <span>{props.initials}</span>
);

export interface UserAvatarProps {
  user: User;
}

export const UserAvatar: Component<UserAvatarProps> = (props) => { /* … */ };
```

## Declare Components as Typed Consts

Declare components as `const PascalCase: Component<Props> = (props) => …` using the component types from `"solid-js"`, not as plain function declarations. A `const PascalCase: Component<…>` (or `ParentComponent<…>`) is immediately recognizable as a component when scanning a file, where a `function` declaration reads like any other function until you find its JSX. The annotation also states the children contract explicitly: `Component` for none expected, `ParentComponent` for optional children, `VoidComponent` to forbid them (see the TypeScript rules).

```tsx
// Bad: reads like a regular function; children contract unstated
export function UserCard(props: UserCardProps) {
  return <div class="card">{props.user.name}</div>;
}

// Good
import type { Component } from "solid-js";

export interface UserCardProps {
  user: User;
}

export const UserCard: Component<UserCardProps> = (props) => (
  <div class="card">{props.user.name}</div>
);
```

## Name Props `<ComponentName>Props`

The props type is the component name in PascalCase suffixed with `Props`, whether declared as an interface or a type alias. Generic names hide which component a type belongs to and collide when files merge.

```tsx
// Bad
interface Props {
  user: User;
}
type CardData = { user: User };

// Good
interface UserCardProps {
  user: User;
}

export const UserCard: Component<UserCardProps> = (props) => { /* … */ };
```
