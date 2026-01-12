# Component Standards

## Module Structure

The frontend is organized into feature-based modules under `src/modules/`:

```
src/modules/
├── ui/                              # Shared UI primitives
│   ├── components/
│   │   ├── Badge.tsx
│   │   ├── Button.tsx
│   │   └── ...
│   ├── ui.animations.ts
│   └── ui.styles.ts
│
├── resources/                       # Resource tracking domain
│   ├── components/
│   │   ├── StaminaCard.tsx
│   │   └── ...
│   ├── resources.query.ts
│   ├── resources.atoms.ts
│   ├── resources.types.ts
│   └── resources.utils.ts
│
├── settings/                        # Configuration domain
│   ├── components/
│   │   └── ...
│   ├── settings.query.ts
│   ├── settings.atoms.ts
│   └── settings.types.ts
│
├── games/                           # Game metadata
│   ├── games.types.ts
│   └── games.constants.ts
│
└── core/                            # App-wide infrastructure
    ├── core.queryClient.ts
    └── core.tick.ts
```

## File Naming Convention

| Suffix | Purpose | Example |
|--------|---------|---------|
| `*.atoms.ts` | Jotai atoms | `resources.atoms.ts` |
| `*.query.ts` | TanStack Query options | `resources.query.ts` |
| `*.types.ts` | TypeScript types | `resources.types.ts` |
| `*.utils.ts` | Pure utility functions | `resources.utils.ts` |
| `*.constants.ts` | Constants | `games.constants.ts` |
| `*.styles.ts` | Style utilities | `ui.styles.ts` |
| `*.animations.ts` | Animation variants | `ui.animations.ts` |
| `*.tsx` (no suffix) | React components | `StaminaCard.tsx` |

## Import Rules

**Always use `@/` path aliases** — never relative imports:

```tsx
// Correct - full qualified paths
import { Button } from "@/modules/ui/components/Button";
import { resourcesQueryAtom } from "@/modules/resources/resources.atoms";
import type { GameId } from "@/modules/games/games.types";

// Wrong - no barrel exports
import { Button } from "@/modules/ui";
import { Button } from "@/modules/ui/components";

// Wrong - no relative imports
import { Button } from "../../modules/ui/components/Button";
```

**No barrel exports** — import directly from file path for:
- Explicit dependencies
- Better tree-shaking
- Clearer import tracing

## Domain Component Organization

Domain components live with their module, not in a separate `components/` tree:

```tsx
// Good: Component with its domain
import { StaminaCard } from "@/modules/resources/components/StaminaCard";

// This component has access to:
// - @/modules/resources/resources.types
// - @/modules/resources/resources.utils
// - @/modules/resources/resources.atoms
```

## Checklist

- [ ] Component lives in appropriate module (`ui/` for primitives, domain module for domain)
- [ ] File named in PascalCase matching component name
- [ ] All imports use `@/modules/*` aliases
- [ ] No relative imports
- [ ] No barrel exports
