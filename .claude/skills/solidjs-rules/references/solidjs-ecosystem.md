---
paths: **/*.{ts,tsx,js,jsx}
description: "Blessed SolidJS 1.x library stack; the TanStack Solid adapters, stable-line version pinning against Solid 2.0 betas, solid-ui vendored components on Kobalte + corvu, Paraglide JS i18n, and cross-cutting adapter conventions."
---

# Ecosystem

## Target SolidJS 1.x

`solid-js` 1.9.x is the current stable; 2.0 is beta with no RC. Target 1.x APIs everywhere and do not adopt 2.0-only patterns until this pack says otherwise. This has a concrete package consequence: use the stable TanStack adapter lines — `@tanstack/solid-query` v5, `@tanstack/solid-router` v1, `@tanstack/solid-start` 1.x — whose peer ranges pin Solid 1.x. The `solid-query` v6, `solid-router` v2, and `solid-start` v2 beta lines are Solid-2-only; never mix them into a 1.x app.

## The TanStack Suite Is the Default

For every concern below, reach for the TanStack Solid adapter before any alternative — including over `@solidjs/router` and SolidStart (`@solidjs/start`). Alpha/beta maturity is accepted here by policy — we work on the bleeding edge — but pin versions and read release notes when bumping.

| Concern | Package | Entry points | Maturity |
| --- | --- | --- | --- |
| Server state, caching | `@tanstack/solid-query` | `useQuery`, `useMutation`, `queryOptions`, `mutationOptions` (see data fetching rules) | stable (v5) |
| Routing | `@tanstack/solid-router` | `createFileRoute`, loaders, typed params/search | stable (v1) |
| Full-stack, SSR | `@tanstack/solid-start` | `createServerFn` | RC |
| Forms | `@tanstack/solid-form` | `createForm`, `formOptions` (see forms rules) | stable (v1) |
| Tables, datagrids | `@tanstack/solid-table` | `createSolidTable`, `flexRender` | stable (v8) |
| Virtualized lists | `@tanstack/solid-virtual` | `createVirtualizer`, `createWindowVirtualizer` | stable (v3) |
| Debounce, throttle, queue | `@tanstack/solid-pacer` | `createDebouncedSignal`, `createThrottler`, async variants | beta |
| Client-first sync, live queries | `@tanstack/solid-db` | `createCollection`, `useLiveQuery` | beta |
| Keyboard shortcuts | `@tanstack/solid-hotkeys` | `createHotkey`, `createHotkeySequence`, `HotkeysProvider` | alpha |
| AI chat and agents | `@tanstack/ai-solid` | `useChat` (accessor-shaped: `messages()`, `isLoading()`) | beta |
| Devtools shell | `@tanstack/solid-devtools` | `<TanStackDevtools plugins={…} />` hosting per-library panels | alpha |

`@tanstack/solid-charts` exists but is pre-alpha with an unstable API; do not adopt it without an explicit decision.

## UI Components: solid-ui on Kobalte + corvu

Application UI components come from solid-ui's copy-paste registry, vendored into the repo (conventionally `src/components/ui`) and styled with Tailwind CSS, which is the styling norm. Underneath, the behavior and accessibility layer is `@kobalte/core` — the Solid analog of Base UI or React Aria Components — with scoped `@corvu/*` packages filling gaps (drawer, resizable, OTP field, calendar). The ownership split is deliberate: Kobalte and corvu stay npm dependencies maintained upstream; the vendored styling and composition layer is project code — edit it freely, and maintain it like any other code, because upstream fixes do not arrive automatically.

- Vendored UI components are views. Keep them dumb per the state architecture rules; no business logic under `components/ui`.
- Build new interactive components on Kobalte primitives rather than hand-rolling ARIA behavior; purely presentational pieces need no foundation.
- Do not copy solid-ui's date picker — it is the registry's one Ark UI component and upstream is replacing it. Compose a picker from `@corvu/calendar` and a Kobalte popover instead.
- `@ark-ui/solid` (Zag.js) is an acceptable alternative headless base when its wider widget catalog is decisive; one foundation per app, never a mix.
- Icons: `lucide-solid`.

## Adapter Conventions: Options In as Functions, Reactivity Out

The Solid adapters share conventions that differ from their React counterparts; getting these wrong silently breaks reactivity.

- Reactive options are passed as accessor functions: `useQuery(() => ({ queryKey: ["todos", filter()], … }))`, `createForm(() => ({ … }))`. Signals read inside re-trigger the library.
- Reactive data crossing into a plain options object goes through a getter: `createSolidTable({ get data() { return rows() }, columns })`. Passing `data: rows()` snapshots the signal once.
- Returned objects are fine-grained stores or accessors: read `query.data` and `query.isPending` as properties inside tracking scopes, call `field()` and `Route.useLoaderData()()` where the adapter returns accessors, and never destructure results.
- Read signals inside library callbacks, not before them: `useLiveQuery((q) => q.from({ todos: todoCollection }).where(({ todos }) => gt(todos.priority, minPriority())))` re-runs when `minPriority` changes; hoisting the read outside the callback freezes it.
- Prefer the `use*` entry points in solid-query; the `create*` names are legacy aliases.

## Internationalization: Paraglide JS

`@inlang/paraglide-js` (2.x) is the i18n library. It is compiler-first: messages compile to tree-shakeable, fully typed functions (`m.greeting({ name })`), so type safety comes from codegen rather than runtime lookups, and the runtime is framework-agnostic — no Solid adapter is needed beyond `paraglideVitePlugin`.

- Messages are plain typed functions, callable from components and domain modules alike; there is no hook to thread through the UI layer.
- With TanStack Start, follow the TanStack Router repo's `start-i18n-paraglide` Solid example: `paraglideMiddleware` on the server, router `rewrite` with `localizeUrl`/`deLocalizeUrl`, and a strategy array like `["url", "cookie", "preferredLanguage", "baseLocale"]`.
- Strategy configuration, locale switching, and custom locale strategies are covered in the i18n rules.
- Do not use solid-i18next (archived upstream). `@solid-primitives/i18n` is the fallback only when instant in-app locale switching with fine-grained reactivity outweighs generated message types; its usage patterns are in the i18n rules.

## Do Not Confuse the Stores

`@tanstack/solid-store` (alpha) is the state engine underneath other TanStack libraries, not our application state layer — business state uses `solid-js/store` per the state architecture rules. When a TanStack library hands you one of its Store instances, subscribe with `useSelector(store, selector)`; its `useStore` is deprecated in the Solid adapter.
