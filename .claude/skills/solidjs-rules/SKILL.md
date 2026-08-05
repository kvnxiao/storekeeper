---
name: solidjs-rules
description: "Use for SolidJS implementation, review, and architecture decisions: reactivity, components and props, component file conventions, control flow, stores and state, state architecture and business logic separation, ecosystem and library choices, data fetching, forms, lifecycle and refs, testing, TypeScript, and React-to-Solid migration."
paths: "**/*.tsx,**/*.jsx,**/*.ts,**/*.js"
---

# SolidJS Rules

Use for SolidJS implementation, review, and architecture decisions: reactivity, components and props, component file conventions, control flow, stores and state, state architecture and business logic separation, ecosystem and library choices, data fetching, forms, lifecycle and refs, testing, TypeScript, and React-to-Solid migration.

## Rule References

- [Reactivity](references/solidjs-reactivity.md): Read when creating or reviewing signals, memos, effects, derived state, batch/untrack, or reactive ownership.
- [Components and props](references/solidjs-components-and-props.md): Read when writing or reviewing components, props handling, defaults, prop forwarding, or children.
- [Component conventions](references/solidjs-component-conventions.md): Read when creating or organizing component files: exports per file, component declaration style, and props type naming.
- [Control flow](references/solidjs-control-flow.md): Read when rendering conditionals or lists in JSX, or choosing between Show, For, Index, Switch, Dynamic, and Portal.
- [Stores and state](references/solidjs-stores-and-state.md): Read when managing nested or shared state with createStore, produce, reconcile, context, or global state.
- [State architecture](references/solidjs-state-architecture.md): Read when deciding where state and business logic live: state modules, actions and selectors, or component-local UI state.
- [Ecosystem](references/solidjs-ecosystem.md): Read when choosing or adding libraries, targeting Solid versions, or setting up routing, SSR, headless UI, tables, virtualization, timing utilities, hotkeys, i18n, or devtools.
- [Data fetching](references/solidjs-data-fetching.md): Read when fetching or mutating server state with TanStack Query, wiring Suspense and ErrorBoundary, integrating router loaders, or using createResource.
- [Forms](references/solidjs-forms.md): Read when building or reviewing forms, validation schemas, or form submission flows.
- [Lifecycle and refs](references/solidjs-lifecycle-and-refs.md): Read when using onMount, onCleanup, element refs, or integrating imperative and third-party code.
- [Testing](references/solidjs-testing.md): Read when adding or reviewing tests for SolidJS components, primitives, or reactive behavior.
- [TypeScript](references/solidjs-typescript.md): Read when typing components, props, signals, events, or refs, or configuring TypeScript for Solid.
- [React to Solid migration](references/solidjs-react-migration.md): Read when porting React code or React habits to SolidJS, or reviewing Solid code written by React developers.
