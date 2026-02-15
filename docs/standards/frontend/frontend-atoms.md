# Jotai Atoms

State management uses [Jotai](https://jotai.org/) with `jotai-tanstack-query` for server state and `jotai-effect` for side effects.

## AtomContainer Pattern

**Never declare atoms at global scope.** All atoms must be declared inside an `*Atoms` class (the "AtomContainer") in `*.atoms.ts` files. Only a single global `atoms` const is exported from `modules/atoms.ts` for namespaced access.

### Why

- Namespacing — `atoms.core.tick` vs a loose `tickAtom` avoids naming collisions
- Encapsulation — private atoms stay `private` on the class, unreachable from components
- Dependency injection — child containers receive parent containers via constructor (e.g., `GenshinAtoms` receives `CoreAtoms`)

### Container Class Structure

Each `*.atoms.ts` file exports a single class:

```tsx
// modules/settings/settings.atoms.ts
import { atom } from "jotai";
import { atomWithQuery } from "jotai-tanstack-query";
import { secretsQueryOptions } from "@/modules/settings/settings.query";

export class SettingsAtoms {
  readonly secretsQuery = atomWithQuery(() => secretsQueryOptions());

  /** Private atoms are not accessible outside the class */
  private readonly originalConfig = atom<AppConfig | null>(null);

  readonly editedConfig = atom<AppConfig | null>(null);

  readonly isDirty = atom((get) => {
    const edited = get(this.editedConfig);
    const original = get(this.originalConfig);
    return !deepEqual(edited, original);
  });
}
```

Child containers that depend on shared atoms accept them via constructor:

```tsx
// modules/games/genshin/genshin.atoms.ts
import type { CoreAtoms } from "@/modules/core/core.atoms";

export class GenshinAtoms {
  constructor(readonly core: CoreAtoms) {}

  readonly resin = atom((get) => {
    const { data } = get(this.core.resourcesQuery);
    return data?.games?.GENSHIN_IMPACT?.find((r) => r.type === "resin") ?? null;
  });
}
```

### Root Container & Global Export

All atom containers are assembled in `modules/atoms.ts` and a single `atoms` const is exported:

```tsx
// modules/atoms.ts
import { CoreAtoms } from "@/modules/core/core.atoms";
import { GenshinAtoms } from "@/modules/games/genshin/genshin.atoms";
import { SettingsAtoms } from "@/modules/settings/settings.atoms";

class GamesAtoms {
  readonly genshin: GenshinAtoms;
  constructor(core: CoreAtoms) {
    this.genshin = new GenshinAtoms(core);
  }
}

class AtomsContainer {
  readonly core: CoreAtoms;
  readonly games: GamesAtoms;
  readonly settings: SettingsAtoms;

  constructor() {
    this.core = new CoreAtoms();
    this.games = new GamesAtoms(this.core);
    this.settings = new SettingsAtoms();
  }
}

export const atoms = new AtomsContainer();
```

### Usage in Components

Always access atoms through the `atoms` namespace:

```tsx
import { useAtomValue } from "jotai";
import { atoms } from "@/modules/atoms";

function StaminaCard() {
  const tick = useAtomValue(atoms.core.tick);
  const resin = useAtomValue(atoms.games.genshin.resin);
  // ...
}
```

## Effect Atoms

Use `atomEffect` for side effects with cleanup. Keep the effect private and expose a readonly derived atom:

```tsx
export class CoreAtoms {
  private readonly tickBase = atom<number>(Date.now());

  private readonly tickEffect = atomEffect((_get, set) => {
    const interval = setInterval(() => set(this.tickBase, Date.now()), 60_000);
    return () => clearInterval(interval);
  });

  readonly tick = atom((get) => {
    get(this.tickEffect);
    return get(this.tickBase);
  });
}
```

## Action Atoms

Complex business logic belongs in action atoms, not component callbacks:

```tsx
// Bad: Complex orchestration in component
const handleSave = useCallback(async () => {
  await Promise.all([saveConfig(config), saveSecrets(secrets)]);
  await reloadConfig();
}, [config, secrets]);

// Good: Action atom inside the container class
export class SettingsAtoms {
  readonly save = atom(null, async (get, set) => {
    const config = get(this.editedConfig);
    const secrets = get(this.editedSecrets);
    if (!config || !secrets) return;

    const { mutateAsync: doSaveConfig } = get(this.saveConfigMutation);
    const { mutateAsync: doSaveSecrets } = get(this.saveSecretsMutation);

    await Promise.all([doSaveConfig(config), doSaveSecrets(secrets)]);
    set(this.markAsSaved);
  });
}

// Component becomes simple
const saveSettings = useSetAtom(atoms.settings.save);
<Button onPress={() => void saveSettings()}>Save</Button>
```

## What Components Should NOT Contain

Move these patterns to atoms:

- Long async orchestration
- Direct `invoke()` or `listen()` calls
- `useEffect` for event subscriptions

Components should only contain:

- Simple state updates
- UI event handlers that delegate to atoms
- Memoized derived values for rendering

## Checklist

- [ ] All atoms declared inside an `*Atoms` class, never at global scope
- [ ] Only `atoms` const exported from `modules/atoms.ts`
- [ ] Components access atoms via `atoms.core.*`, `atoms.games.genshin.*`, etc.
- [ ] Private atoms use `private readonly`
- [ ] Child containers receive dependencies via constructor
- [ ] Effect atoms are private, exposed through a readonly derived atom
- [ ] Complex business logic in action atoms, not components
