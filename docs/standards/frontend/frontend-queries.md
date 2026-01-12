# Query Options Pattern

TanStack Query options are defined in `*.query.ts` files with direct Tauri imports.

**Reference docs:**
- [queryOptions](https://tanstack.com/query/v5/docs/framework/react/reference/queryOptions)
- [mutationOptions](https://tanstack.com/query/latest/docs/framework/react/reference/mutationOptions)

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

Export functions that return `queryOptions()` for dependency injection flexibility:

```tsx
// modules/resources/resources.query.ts
import { queryOptions } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { AllResources } from "@/modules/resources/resources.types";

export function resourcesQueryOptions() {
  return queryOptions({
    queryKey: ["resources"],
    queryFn: async () => invoke<AllResources>("get_all_resources"),
    retry: false,
    refetchOnWindowFocus: true,
  });
}
```

## Mutation Options

Export functions that return `mutationOptions()`:

```tsx
import { mutationOptions } from "@tanstack/react-query";

export function refreshResourcesMutationOptions() {
  return mutationOptions({
    mutationKey: ["refresh-resources"],
    mutationFn: async () => invoke<AllResources>("refresh_resources"),
  });
}
```

## Usage in Atoms

Call the functions when creating atoms:

```tsx
// modules/resources/resources.atoms.ts
import { atomWithQuery, atomWithMutation } from "jotai-tanstack-query";
import { resourcesQueryOptions, refreshResourcesMutationOptions } from "./resources.query";

export const resourcesQueryAtom = atomWithQuery(() => resourcesQueryOptions());
export const refreshResourcesMutationAtom = atomWithMutation(() => refreshResourcesMutationOptions());
```

## Key Points

- **Always export functions**, not constants — enables future dependency injection
- Use `queryOptions()` and `mutationOptions()` helper functions inside the exported functions
- Import `invoke` directly from `@tauri-apps/api/core`
- No Jotai atoms in query files — those belong in `*.atoms.ts`
- Query keys should be descriptive arrays

## Checklist

- [ ] Query/mutation options in `*.query.ts` files as **exported functions**
- [ ] Functions return `queryOptions()` / `mutationOptions()` calls
- [ ] Direct Tauri `invoke` imports
- [ ] No atoms in query files
