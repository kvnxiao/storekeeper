---
paths: **/*.{ts,tsx,js,jsx}
description: "SolidJS server-state rules; TanStack Query with queryOptions factories in domain modules, no destructuring of query results, Suspense/ErrorBoundary composition, mutations as domain options, router loader integration, and createResource as the low-level fallback."
---

# Data Fetching

## TanStack Query Owns Server State

Server data lives in the Query cache — not fetched ad hoc in components, and never copied into stores. Raw `fetch` in an effect loses tracking after the first `await` and handles neither races, caching, deduplication, nor retries. `useQuery` from `@tanstack/solid-query` is the default for reads; its `data` is backed by a Solid resource, so Suspense and transitions work out of the box.

```tsx
// Bad
createEffect(async () => setUser(await fetchUser(userId())));

// Good
const user = useQuery(() => userQueryOptions(userId()));
```

## Define `queryOptions` in Domain Modules

Query keys, fetchers, and staleness policy are business decisions; house them in the domain module (see the state architecture rules) as `queryOptions` factories. Components, router loaders, and `queryClient` calls all consume the same factory, so keys can never drift.

```ts
// src/state/todos.ts
import { queryOptions } from "@tanstack/solid-query";

export function todosQueryOptions(filter: TodoFilter) {
  return queryOptions({
    queryKey: ["todos", filter],
    queryFn: () => api.fetchTodos(filter),
    staleTime: 5 * 60 * 1000,
  });
}
```

## Options In as a Function, Results Out Fine-Grained

This is the pack-wide adapter convention (see the ecosystem rules); for Query specifically: `useQuery` takes an accessor returning options — signals read inside it are tracked, and changes re-key or re-run the query. Gate dependent queries with `enabled` instead of conditional calls. The result is a fine-grained store: read `query.data`, `query.isPending`, `query.isError` as properties inside tracking scopes, and never destructure it. Being store-backed also means `query.data` is a proxy — `unwrap` it before cloning, serializing, or sending it across IPC (see the stores and state rules).

```tsx
const [todo, setTodo] = createSignal(0);

const todoQuery = useQuery(() => ({
  ...todoQueryOptions(todo()),
  enabled: todo() > 0,
}));

// Bad: destructuring severs reactivity, exactly like props and stores
const { data, isPending } = useQuery(() => todosQueryOptions("all"));
```

## Compose `ErrorBoundary` Outside, `Suspense` Inside

Reading `query.data` under a `Suspense` boundary triggers the fallback while loading. Set `throwOnError: true` to surface fetch errors to the nearest `ErrorBoundary`; otherwise render states explicitly with `<Switch>` on `isPending`/`isError`.

```tsx
<ErrorBoundary fallback={<p>Couldn't load todos.</p>}>
  <Suspense fallback={<p>Loading…</p>}>
    <For each={todos.data}>{(todo) => <TodoRow todo={todo} />}</For>
  </Suspense>
</ErrorBoundary>
```

## Mutations Are Domain Logic

`useMutation` also takes function-wrapped options. What a mutation does — the request, optimistic update, rollback, and which queries it invalidates — is business logic and belongs in the domain module via `mutationOptions`; the component only calls `mutation.mutate`.

```ts
// src/state/todos.ts
import { mutationOptions, type QueryClient } from "@tanstack/solid-query";

export function addTodoMutationOptions(queryClient: QueryClient) {
  return mutationOptions({
    mutationFn: (todo: NewTodo) => api.addTodo(todo),
    onSettled: () =>
      queryClient.invalidateQueries({ queryKey: ["todos"] }),
  });
}
```

```tsx
const queryClient = useQueryClient();
const addTodo = useMutation(() => addTodoMutationOptions(queryClient));

<button onClick={() => addTodo.mutate(draft())}>Add</button>
```

## Integrate the Router Through the Cache

Put the `QueryClient` in router context and warm the cache in loaders with `ensureQueryData`; the component subscribes with `useQuery` on the same options factory. With `defaultPreload: "intent"`, hover and focus start fetching before navigation.

```tsx
export const Route = createFileRoute("/todos")({
  loader: ({ context: { queryClient } }) =>
    queryClient.ensureQueryData(todosQueryOptions("all")),
  component: TodosPage,
});

const TodosPage: Component = () => {
  const todos = useQuery(() => todosQueryOptions("all"));
  return <For each={todos.data}>{(todo) => <TodoRow todo={todo} />}</For>;
};
```

Route hooks return accessors in the Solid adapter — `Route.useParams()`, `Route.useSearch()`, and `Route.useLoaderData()` are called as functions (`params().postId`).

## Split Code at Routes, Transition Between States

Route-level code splitting uses TanStack Router's lazy route files: move a route's component into `posts.lazy.tsx` with `createLazyFileRoute` while the loader and route config stay eager. For component-level splitting, `lazy(() => import("./HeavyEditor"))` from `solid-js` renders under the same `Suspense` boundaries as data — the `React.lazy` habit maps directly. When a signal change swaps Suspense-bound content (tab switches, filter changes), wrap the write in `useTransition` from `solid-js` to keep the current UI visible instead of flashing fallbacks.

```tsx
const [pending, start] = useTransition();
<button onClick={() => start(() => setTab("stats"))} data-pending={pending()}>
  Stats
</button>
```

## `createResource` Is the Low-Level Fallback

For library code and contexts without a `QueryClient`, `createResource(source, fetcher)` remains correct: a source of `null`/`undefined`/`false` skips the fetcher, changes re-run it, and `data.loading`/`data.error` plus `mutate`/`refetch` cover local needs. It is the primitive solid-query itself builds on — use it when pulling in the full cache is unjustified, not as the default for application server state.
