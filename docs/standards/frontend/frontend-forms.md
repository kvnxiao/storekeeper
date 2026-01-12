# Form State Management

Patterns for managing form state with Jotai atoms.

## Dirty State Tracking

Track form modifications with deep equality:

```tsx
import { deepEqual } from "fast-equals";
import { atom } from "jotai";

const originalConfigAtom = atom<AppConfig | null>(null);
const editedConfigAtom = atom<AppConfig | null>(null);

export const isDirtyAtom = atom((get) => {
  const edited = get(editedConfigAtom);
  const original = get(originalConfigAtom);
  if (!edited || !original) return false;
  return !deepEqual(edited, original);
});

export const markAsSavedAtom = atom(null, (get, set) => {
  const current = get(editedConfigAtom);
  if (current) {
    set(originalConfigAtom, structuredClone(current));
  }
});
```

**Important**: Use `structuredClone()` for deep copies, not spread operator.

## Form Initialization

Initialize form state from query data:

```tsx
import { atomEffect } from "jotai-effect";

const formInitEffect = atomEffect((get, set) => {
  const { data: loadedConfig } = get(configQueryAtom);
  const { data: loadedSecrets } = get(secretsQueryAtom);
  const currentConfig = get(editedConfigAtom);

  // Only initialize if queries loaded and form not already initialized
  if (loadedConfig && loadedSecrets && !currentConfig) {
    set(editedConfigAtom, loadedConfig);
    set(originalConfigAtom, structuredClone(loadedConfig));
  }
});

export const formInitAtom = atom((get) => {
  get(formInitEffect);
});

// Subscribe in component
const SettingsPage = () => {
  useAtomValue(formInitAtom);
  // ...
};
```

## Module Organization

```
modules/settings/
├── settings.query.ts        # Query/mutation options
├── settings.atoms.ts        # Form state + actions
├── settings.types.ts        # TypeScript interfaces
└── components/
    └── GeneralSection.tsx   # Uses atoms for state
```

## Checklist

- [ ] Dirty tracking uses `deepEqual` from `fast-equals`
- [ ] Deep copies use `structuredClone()`, not spread
- [ ] Form initialization via `atomEffect`
- [ ] Init effect checks for existing state before overwriting
