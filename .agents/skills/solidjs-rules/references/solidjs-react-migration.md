---
paths: **/*.{tsx,jsx}
description: "React-to-Solid migration corrections; the React habits that silently break in SolidJS and their Solid idioms, plus eslint-plugin-solid guardrails."
---

# React to Solid Migration

Solid's JSX looks like React, but its execution model differs: components run once and updates flow through fine-grained signals, not re-renders. The table maps React habits that break Solid reactivity to Solid patterns. The other rules in this pack cover the patterns in depth.

## Rendering Model

| React habit | Solid correction |
| --- | --- |
| Expecting the component to re-run on state change | It never re-runs; put per-update logic in JSX, derived functions, memos, or effects |
| Early return for loading/empty states | `<Show>` / `<Switch>` in JSX |
| `{count}` in JSX, `console.log(count)` | Signal getters are functions: `{count()}`, `console.log(count())` |
| Destructuring props (`const { name } = props`) | Access `props.name` at point of use; `splitProps` / `mergeProps` |
| Default-parameter destructuring for prop defaults | `mergeProps({ size: "md" }, props)` |
| `key` props on lists, `.map()` in JSX | `<For>` (keys by reference) or `<Index>` (fixed slots) |

## Reactivity

| React habit | Solid correction |
| --- | --- |
| Dependency arrays on effects/memos | Auto-tracked; use `on()` for explicit or deferred dependencies |
| `useEffect` to derive state from state | Derive: `const b = () => f(a())` |
| Async effects for data fetching | TanStack Query (`useQuery`) or `createResource`; tracking breaks after `await` |
| Returning a cleanup function from an effect | `onCleanup(fn)` inside the effect |
| `useCallback` / `useMemo` / `React.memo` for referential stability | Unnecessary; closures are stable because components run once, and equal signal writes do not propagate |
| Rules-of-hooks contortions (no conditionals, top-level only) | Primitives may be created in conditionals, loops, or outside components (given an owner) |
| `useRef` as a mutable instance box | A plain `let` in the component body |

## DOM and Events

| React habit | Solid correction |
| --- | --- |
| `className`, `htmlFor` | `class`, `for`; `classList={{ active: isActive() }}` for conditional classes |
| `onChange` firing per keystroke | Native semantics: `onInput` per keystroke, `onChange` on commit/blur |
| `style={{ marginTop: 8 }}` | CSS property names with units: `style={{ "margin-top": "8px" }}` |
| Controlled-input re-render loop | `value={text()}` + `onInput` works directly; there is no re-render to fight |

## State and Children

| React habit | Solid correction |
| --- | --- |
| Storing a function in state (`setFn(fn)`) | Setters treat functions as updaters: `setFn(() => fn)` |
| Context with a default value as fallback | No default; expose a hook that throws when the provider is missing |
| Reading `props.children` repeatedly or inspecting it as data | Resolve once with the `children()` helper |
| Expecting remount when a value changes | Non-keyed `<Show>` preserves the branch; use `keyed` for remount-on-identity-change |

## Guardrails (Default)

Default migrations to `eslint-plugin-solid`. Its `reactivity`, `no-destructure`, `prefer-for`, `components-return-once`, `no-react-deps`, and `no-react-specific-props` rules catch the incompatible React patterns in this table. Keep an existing lint stack when it enforces equivalent checks.
