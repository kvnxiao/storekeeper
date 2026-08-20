---
paths: **/*.{ts,tsx,js,jsx}
description: "SolidJS 1.x ecosystem policy; compatibility boundaries, stable-first TanStack and UI preferences, adapter reactivity, Paraglide JS, and store distinctions."
---

# Ecosystem

## Target SolidJS 1.x (Required)

This rule pack targets SolidJS 1.x. Do not mix Solid-2-only package lines or APIs into a 1.x application. The `@tanstack/solid-query` v6, `@tanstack/solid-router` v2, and `@tanstack/solid-start` v2 lines target Solid 2; use their preceding Solid-1-compatible lines in applications governed by this pack.

## The TanStack Suite Is the Default (Default)

For projects using this rule pack, default to the TanStack Solid adapter for the concerns below. Use another stable package when an existing stack, compatibility boundary, or missing capability makes it a better fit.

| Concern | Package | Entry points | Maturity tier |
| --- | --- | --- | --- |
| Server state, caching | `@tanstack/solid-query` | `useQuery`, `useMutation`, `queryOptions`, `mutationOptions` | Stable |
| Routing | `@tanstack/solid-router` | `createFileRoute`, loaders, typed params and search | Stable |
| Full-stack, SSR | `@tanstack/solid-start` | `createServerFn` | Release candidate |
| Forms | `@tanstack/solid-form` | `createForm`, `formOptions` | Stable |
| Tables, data grids | `@tanstack/solid-table` | `createSolidTable`, `flexRender` | Stable |
| Virtualized lists | `@tanstack/solid-virtual` | `createVirtualizer`, `createWindowVirtualizer` | Stable |
| Debounce, throttle, queue | `@tanstack/solid-pacer` | `createDebouncedSignal`, `createThrottler` | Beta |
| Client-first sync, live queries | `@tanstack/solid-db` | `createCollection`, `useLiveQuery` | Beta |
| Keyboard shortcuts | `@tanstack/solid-hotkeys` | `createHotkey`, `createHotkeySequence` | Alpha |
| AI chat and agents | `@tanstack/ai-solid` | `useChat` | Beta |
| Devtools shell | `@tanstack/solid-devtools` | `TanStackDevtools` | Alpha |

Stable packages are the default. A prerelease dependency needs an exact version, a named owner, an update policy, and an exit condition recorded in project documentation. A pre-alpha package such as `@tanstack/solid-charts` requires an explicit architecture decision.

## UI Components: solid-ui on Kobalte and Corvu (Default)

For projects using this rule pack, default to solid-ui's copy-paste registry, vendored under `src/components/ui` and styled with Tailwind CSS. Use `@kobalte/core` for behavior and accessibility, and use scoped `@corvu/*` packages for capabilities such as drawers, resizable panes, OTP fields, and calendars.

- Vendored UI components remain views and contain no business logic.
- Default new interactive components to Kobalte primitives; purely presentational components need no headless foundation.
- `@ark-ui/solid` is an acceptable alternative when its widget catalog is decisive. Prefer one headless foundation per application unless a missing component justifies an exception.
- Default icons to `lucide-solid` unless the project already has an icon system.

Vendored registry code is project code, so upstream fixes require a deliberate update.

## Adapter Conventions: Options In as Functions, Reactivity Out (Required)

Solid adapters must receive reactive options through accessors or property getters, and consumers must read returned stores and accessors without destructuring.

- Pass reactive options as accessor functions: `useQuery(() => ({ queryKey: ["todos", filter()] }))` and `createForm(() => ({ ...options }))`.
- Pass reactive data through getters in plain option objects: `createSolidTable({ get data() { return rows() }, columns })`.
- Read returned stores as properties inside tracking scopes, and call accessors such as `field()` and `Route.useLoaderData()()`.
- Read signals inside adapter callbacks; a read hoisted outside the callback freezes the value.
- Use the `use*` entry points in solid-query; the `create*` names are legacy aliases.

## Internationalization: Paraglide JS (Default)

Default to `@inlang/paraglide-js` for compiled, typed messages and framework-independent runtime behavior. Keep its v2 strategy API when the application depends on ordered locale strategies.

- With TanStack Start, use `paraglideMiddleware`, router rewrites with `localizeUrl` and `deLocalizeUrl`, and an ordered strategy array.
- Call compiled message functions from components or domain modules without a UI hook.
- Use `@solid-primitives/i18n` when instant in-app locale switching with fine-grained reactivity matters more than generated message types.
- The archived `solid-i18next` repository is not a maintained choice.

## Distinguish application and TanStack stores (Default)

Use `solid-js/store` for application state. `@tanstack/solid-store` is the state engine used by TanStack libraries; when a library returns one of its stores, subscribe with `useSelector(store, selector)`. Its Solid adapter retains `useStore` only as a deprecated alias.
