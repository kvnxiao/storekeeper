# Query Options Pattern

TanStack Query options are defined in `*.query.ts` files with direct Tauri imports.

## File Structure

```
src/modules/resources/
├── resources.query.ts    # Query options with direct Tauri invoke
└── resources.atoms.ts    # Atoms wrapping queries

src/modules/settings/
├── settings.query.ts     # Query/mutation options
└── settings.atoms.ts     # Atoms wrapping queries
```

## Query Options

```tsx
// modules/resources/resources.query.ts
import { invoke } from "@tauri-apps/api/core";
import type { QueryOptions, MutationOptions } from "@tanstack/react-query";
import type { AllResources } from "@/modules/resources/resources.types";

export const resourcesQueryOptions: QueryOptions<AllResources> = {
  queryKey: ["resources"],
  queryFn: async () => invoke<AllResources>("get_all_resources"),
  retry: false,
  refetchOnWindowFocus: true,
};
```

## Mutation Options

```tsx
export const refreshResourcesMutationOptions: MutationOptions<
  AllResources,
  Error,
  void
> = {
  mutationKey: ["refresh-resources"],
  mutationFn: async () => invoke<AllResources>("refresh_resources"),
};
```

## Key Points

- Import `invoke` directly from `@tauri-apps/api/core`
- Export typed `QueryOptions` and `MutationOptions` objects
- No Jotai atoms in query files — those belong in `*.atoms.ts`
- Query keys should be descriptive arrays

## Checklist

- [ ] Query options in `*.query.ts` files
- [ ] Direct Tauri `invoke` imports
- [ ] Typed `QueryOptions<T>` and `MutationOptions<T, E, V>`
- [ ] No atoms in query files
