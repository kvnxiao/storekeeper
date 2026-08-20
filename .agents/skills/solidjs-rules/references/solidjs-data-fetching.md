---
paths: **/*.{ts,tsx,js,jsx}
description: "SolidJS server-state rules; TanStack Query with queryOptions factories in domain modules, no destructuring of query results, Suspense/ErrorBoundary composition, mutations as domain options, router loader integration, and createResource as the low-level fallback."
---

# Data Fetching

## TanStack Query Owns Server State (Default)

When application fetching needs caching, retries, invalidation, deduplication, or preloading, keep server data in the Query cache and use `useQuery` from `@tanstack/solid-query`. Raw `fetch` in an effect loses tracking after the first `await` and supplies none of those policies. A deliberate copy is appropriate for an editable draft, offline snapshot, serialization boundary, or state that must become independent of cache updates.

```tsx
createEffect(async () => setUser(await fetchUser(userId())));
```

The query cache owns the request lifecycle:

```tsx
const user = useQuery(() => userQueryOptions(userId()));
```

## Define `queryOptions` in Domain Modules (Default)

Default query keys, fetchers, and staleness policy to `queryOptions` factories in the domain module. Keep view-specific queries local when they have no reusable domain policy. Reuse a shared factory in components, router loaders, and `queryClient` calls.

```ts
import { queryOptions } from "@tanstack/solid-query";

export function todosQueryOptions(filter: TodoFilter) {
  return queryOptions({
    queryKey: ["todos", filter],
    queryFn: () => api.fetchTodos(filter),
    staleTime: 5 * 60 * 1000,
  });
}
```

## Options In as a Function, Results Out Fine-Grained (Required)

This is the pack-wide adapter convention (see the ecosystem rules). For Query, `useQuery` takes an accessor returning options; signals read inside it are tracked, and changes re-key or re-run the query. Gate dependent queries with `enabled` instead of conditional calls. The result is a fine-grained store: read `query.data`, `query.isPending`, and `query.isError` as properties inside tracking scopes, and never destructure it. Because the result is store-backed, `query.data` is a proxy; call `unwrap` before cloning, serializing, or sending it across IPC (see the stores and state rules).

```tsx
const [todo, setTodo] = createSignal(0);

const todoQuery = useQuery(() => ({
  ...todoQueryOptions(todo()),
  enabled: todo() > 0,
}));
```

Destructuring the result severs reactivity:

```tsx
const { data, isPending } = useQuery(() => todosQueryOptions("all"));
```

## Compose `ErrorBoundary` Outside, `Suspense` Inside (Default)

Reading `query.data` under a `Suspense` boundary triggers the fallback while loading. Default `throwOnError: true` when the surrounding `ErrorBoundary` owns error presentation; otherwise render states explicitly with `<Switch>` on `isPending` and `isError`.

```tsx
<ErrorBoundary fallback={<p>Couldn't load todos.</p>}>
  <Suspense fallback={<p>Loading…</p>}>
    <For each={todos.data}>{(todo) => <TodoRow todo={todo} />}</For>
  </Suspense>
</ErrorBoundary>
```

## Mutations Are Domain Logic (Default)

`useMutation` also takes function-wrapped options. Default the request, optimistic update, rollback, and invalidation policy to a domain `mutationOptions` factory. Keep a mutation local when it affects only transient view state.

```ts
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

## Integrate the Router Through the Cache (Default)

When a router loader preloads cached server state, default the `QueryClient` to router context and call `ensureQueryData` with the same options factory that the component passes to `useQuery`. With `defaultPreload: "intent"`, hover and focus start fetching before navigation.

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

## Split Code at Routes, Transition Between States (Default)

Default route-level code splitting to TanStack Router lazy route files, with loaders and route configuration left eager. For a heavy component below the route, use `lazy(() => import("./HeavyEditor"))`. When a signal change swaps Suspense-bound content, use `useTransition` to preserve the current UI while the new content loads.

```tsx
const [pending, start] = useTransition();
<button onClick={() => start(() => setTab("stats"))} data-pending={pending()}>
  Stats
</button>
```

## `createResource` Is the Low-Level Fallback (Default)

For isolated fetching, library code, and contexts without a `QueryClient`, default to `createResource(source, fetcher)`: a source of `null`, `undefined`, or `false` skips the fetcher; changes re-run it; and `data.loading`, `data.error`, `mutate`, and `refetch` cover local needs. `createResource` is not deprecated. Solid 2 migration replaces it with async computations and `<Loading>`, not `createAsync`; see the [Solid 2 migration guide](https://github.com/solidjs/solid/blob/next/documentation/solid-2.0/MIGRATION.md#createResource--async-computations--loading).
