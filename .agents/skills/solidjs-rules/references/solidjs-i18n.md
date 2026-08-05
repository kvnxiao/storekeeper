---
paths: **/*.{ts,tsx,js,jsx}
description: "SolidJS internationalization rules; Paraglide JS strategy arrays as ordered fallback chains, server-resolvable locale detection, route-level strategy overrides, setLocale-driven switching, custom strategies over runtime overrides, and @solid-primitives/i18n fallback patterns."
---

# Internationalization

`@inlang/paraglide-js` is the i18n library (see the ecosystem rules for the library choice and TanStack Start wiring). These rules cover locale detection and switching, plus the `@solid-primitives/i18n` patterns for surfaces that use the fallback library.

## The Strategy Array Is an Ordered Fallback Chain

`strategy` is evaluated left to right; the first entry that returns a locale wins and the rest are never checked. Two placement rules follow:

- Strategies that always resolve go last. `url` with the default wildcard pattern and `baseLocale` never return `undefined`, so anything after them is dead.
- User choices beat automatic detection. Put persisted-preference strategies (`cookie`, `localStorage`) before `preferredLanguage`; the reverse means the browser language always wins and a manual selection never takes effect.

Never ship an empty array — it is the most common cause of Paraglide's "No locale found" error — and end with `baseLocale` as the safety net. The default for URL-routed apps:

```js
compile({
  project: "./project.inlang",
  outdir: "./src/paraglide",
  strategy: ["url", "cookie", "preferredLanguage", "baseLocale"],
});
```

`globalVariable` is for tests and prototypes only: it leaks across concurrent requests on the server and does not persist across reloads on the client.

## Every Request Must Be Able to Resolve a Locale

- `localStorage` is browser-only; the server skips it. If a persisted preference must affect the initial document request in an SSR app, pair it with `cookie` — otherwise the server falls through to `preferredLanguage`/`url` and can render a locale that differs from the hydrated client.
- `url` only applies to document requests (`Sec-Fetch-Dest: document`). With `strategy: ["url"]` alone, API and RPC requests resolve no locale; include `cookie` or `baseLocale` as well.
- Under the `url` strategy, messages resolve from the current request. Call `m.*()` inside the request context established by `paraglideMiddleware` (route handlers, loaders); a module-scope `m.hello()` has no URL to read and fails with "No locale found".

## Override Strategy per Route

Public pages use URL prefixes; private routes like `/dashboard` are unprefixed, and the default wildcard `url` pattern resolves them to `baseLocale` before any cookie is consulted. Use `routeStrategies` — checked in declaration order, first match wins — to read the cookie there and to exclude API routes from i18n middleware entirely:

```js
strategy: ["url", "cookie", "baseLocale"],
routeStrategies: [
  { match: "/dashboard/:path(.*)?", strategy: ["cookie", "baseLocale"] },
  { match: "/api/:path(.*)?", exclude: true },
],
```

## Switch Locales with `setLocale`, Not Localized Links

`setLocale()` updates the configured strategies and performs a full document navigation by design, keeping `<html lang>`, SSR state, and URLs in sync. Under client-side routing, a plain localized href changes the URL without a document load, so the UI never updates:

```tsx
// Good
<button onClick={() => setLocale("de")}>Deutsch</button>

// Bad: client-side navigation, UI stays in the old locale
<a href={localizeHref("/page", { locale: "de" })}>Deutsch</a>
```

If a link must be used, force a full-document navigation (for example, the router's reload attribute on that link). `setLocale(locale, { reload: false })` plus `overwriteGetLocale(localeSignal)` is a narrow escape hatch for fully client-rendered surfaces whose strategy excludes `url`; do not use it to turn URL-routed locale changes into signal-driven re-renders.

## Prefer Custom Strategies over Runtime Overrides

To read the locale from a nonstandard source (sessionStorage, query param, user database), define a `custom-<name>` strategy and include it in the strategy array instead of reaching for `overwriteGetLocale()`: custom strategies compose with built-ins, fall through to the next entry when they return `undefined`, and isolate errors.

```js
defineCustomClientStrategy("custom-sessionStorage", {
  getLocale: () => sessionStorage.getItem("user-locale") ?? undefined,
  setLocale: (locale) => sessionStorage.setItem("user-locale", locale),
});
// strategy: ["custom-sessionStorage", "cookie", "baseLocale"]
```

Client-side `getLocale` must be synchronous (`setLocale` may be async); server-side `getLocale` may be async for database or auth lookups. Define client strategies in app initialization before first render, and server strategies before the middleware handles requests.

When an override is genuinely required, call `overwriteGetLocale`/`overwriteSetLocale` at the app entrypoint before rendering starts — a forgotten or late call is another "No locale found" source. On the server, the override must be request-scoped via `AsyncLocalStorage` (or the runtime's equivalent); a bare variable races across concurrent requests with different locales.

## `@solid-primitives/i18n` Patterns

When the fallback library is justified (see the ecosystem rules), follow these patterns:

- One dictionary module per locale, typed against the base locale so missing keys fail to compile: `type Dict = typeof en_dict` and `const fr_dict: Dict = { … }`.
- Prefer flat JSON dictionaries for load performance; `i18n.flatten` nested dictionaries once when loading, not per lookup.
- The translator does not resolve `{{ placeholder }}` templates by default; pass `i18n.resolveTemplate` as the second argument.
- Load dictionaries with `createResource` keyed on the locale signal, and either narrow the `undefined` loading state with `Show` or pass the base dictionary as `initialValue` (which bundles it).
- Wrap locale switches in `useTransition` so changing language does not flash Suspense fallbacks. Static dictionaries skip the resource: `createMemo(() => i18n.flatten(dicts[locale()]))`.
- Split large dictionaries per feature module with per-module resources, combined via `i18n.prefix` or scoped via `i18n.scopedTranslator`. For nested-object access prefer `chainedTranslator` over `proxyTranslator` — the proxy has a runtime cost and is mainly useful for mocking translations in tests.

```tsx
const [locale, setLocale] = createSignal<Locale>("en");
const [dict] = createResource(locale, fetchDictionary); // fetcher flattens the dict
const t = i18n.translator(dict, i18n.resolveTemplate);

const [pending, start] = useTransition();
const switchLocale = (l: Locale) => start(() => setLocale(l));
```
