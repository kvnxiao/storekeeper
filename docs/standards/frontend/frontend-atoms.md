# Jotai Atoms

State management uses [Jotai](https://jotai.org/) with `jotai-tanstack-query` for server state and `jotai-effect` for side effects.

## Atom Wrapper Pattern

Wrap query options in atoms in `*.atoms.ts` files:

```tsx
// modules/resources/resources.atoms.ts
import { atom } from "jotai";
import { atomWithMutation, atomWithQuery } from "jotai-tanstack-query";
import {
  refreshResourcesMutationOptions,
  resourcesQueryOptions,
} from "@/modules/resources/resources.query";

export const resourcesQueryAtom = atomWithQuery(() => resourcesQueryOptions);
export const refreshResourcesMutationAtom = atomWithMutation(
  () => refreshResourcesMutationOptions,
);
```

## Naming Convention

Atoms have explicit `Atom` suffix since they're imported directly:

```tsx
import { resourcesQueryAtom, isDirtyAtom, saveAtom } from "@/modules/settings/settings.atoms";
```

## Effect Atoms

Use `atomEffect` for side effects with cleanup:

```tsx
import { listen } from "@tauri-apps/api/event";
import { atomEffect } from "jotai-effect";
import { queryClient } from "@/modules/core/core.queryClient";

const resourcesEventEffect = atomEffect(() => {
  const unlisten = listen<AllResources>("resources-updated", (event) => {
    queryClient.setQueryData(["resources"], event.payload);
  });

  return () => { void unlisten.then((fn) => fn()); };
});

// Expose via derived atom
export const resourcesEventListenerAtom = atom((get) => {
  get(resourcesEventEffect);
});
```

### Interval Effects

```tsx
// modules/core/core.tick.ts
const base = atom<number>(Date.now());

const tickEffect = atomEffect((_get, set) => {
  const interval = setInterval(() => set(base, Date.now()), 60_000);
  return () => clearInterval(interval);
});

export const currentTick = atom((get) => {
  get(tickEffect);
  return get(base);
});
```

## Action Atoms

Complex business logic belongs in action atoms, not component callbacks:

```tsx
// Bad: Complex orchestration in component
const handleSave = useCallback(async () => {
  await Promise.all([saveConfig(config), saveSecrets(secrets)]);
  await reloadConfig();
}, [config, secrets]);

// Good: Action atom encapsulates logic
export const saveAtom = atom(null, async (get, set) => {
  const config = get(editedConfigAtom);
  const secrets = get(editedSecretsAtom);
  if (!config || !secrets) return;

  const { mutateAsync: doSaveConfig } = get(saveConfigMutationAtom);
  const { mutateAsync: doSaveSecrets } = get(saveSecretsMutationAtom);

  await Promise.all([doSaveConfig(config), doSaveSecrets(secrets)]);
  set(markAsSavedAtom);
});

// Component becomes simple
const saveSettings = useSetAtom(saveAtom);
<Button onPress={() => void saveSettings()}>Save</Button>
```

## What Components Should NOT Contain

Move these patterns to atoms:

- ❌ Long async orchestration
- ❌ Direct `invoke()` or `listen()` calls
- ❌ `useEffect` for event subscriptions

Components should only contain:

- ✅ Simple state updates
- ✅ UI event handlers that delegate to atoms
- ✅ Memoized derived values for rendering

## Checklist

- [ ] Atoms in `*.atoms.ts` wrapping query options
- [ ] Atom names have `Atom` suffix
- [ ] Event listeners use `atomEffect` with cleanup
- [ ] Complex business logic in action atoms
- [ ] Private atoms not exported
