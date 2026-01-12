# TypeScript Standards

## Type File Organization

Types are co-located with their domain module in `*.types.ts` files:

```
src/modules/
├── games/
│   └── games.types.ts        # GameId, GameMetadata
├── resources/
│   └── resources.types.ts    # StaminaResource, CooldownResource, AllResources
├── settings/
│   └── settings.types.ts     # AppConfig, SecretsConfig, game configs
└── types/
    └── intl.d.ts             # Global type augmentations only
```

## Naming Conventions

### Backend vs Frontend Property Names

- **Backend config types** (from Rust/TOML): `snake_case` properties
- **Resource types** (transformed by serde): `camelCase` properties
- **Type names**: Always `PascalCase`

```tsx
// Config types - match Rust exactly (snake_case)
// modules/settings/settings.types.ts
export interface GeneralConfig {
  poll_interval_secs: number;
  start_minimized: boolean;
}

// Resource types - serde transforms to camelCase
// modules/resources/resources.types.ts
export interface StaminaResource {
  current: number;
  max: number;
  fullAt: string;           // Rust: full_at
  regenRateSeconds: number; // Rust: regen_rate_seconds
}
```

## Type Guards

Create type guards for discriminated unions with `unknown` input:

```tsx
// modules/resources/resources.types.ts

// Bad: Type assertion without validation
const stamina = data as StaminaResource;

// Good: Type guard with proper checks
export function isStaminaResource(data: unknown): data is StaminaResource {
  return (
    typeof data === "object" &&
    data !== null &&
    "current" in data &&
    "max" in data &&
    "fullAt" in data
  );
}

// Usage - TypeScript narrows the type
if (isStaminaResource(data)) {
  console.log(data.current); // TypeScript knows this is number
}
```

## Literal Union Types

Use union literal types for known string values:

```tsx
// modules/games/games.types.ts

// Bad: String type for known values
interface Game {
  id: string;
}

// Good: Literal union type
export type GameId =
  | "GENSHIN_IMPACT"
  | "HONKAI_STAR_RAIL"
  | "ZENLESS_ZONE_ZERO"
  | "WUTHERING_WAVES";

export interface GameMetadata {
  title: string;
  shortId: string;
}
```

```tsx
// modules/games/games.constants.ts
import type { GameId, GameMetadata } from "@/modules/games/games.types";

// Metadata lookup with full coverage
export const GAME_METADATA: Record<GameId, GameMetadata> = {
  GENSHIN_IMPACT: { title: "Genshin Impact", shortId: "genshin" },
  HONKAI_STAR_RAIL: { title: "Honkai: Star Rail", shortId: "hsr" },
  ZENLESS_ZONE_ZERO: { title: "Zenless Zone Zero", shortId: "zzz" },
  WUTHERING_WAVES: { title: "Wuthering Waves", shortId: "wuwa" },
};

// Ordered array with explicit type
export const GAME_ORDER: GameId[] = [
  "GENSHIN_IMPACT",
  "HONKAI_STAR_RAIL",
  "ZENLESS_ZONE_ZERO",
  "WUTHERING_WAVES",
];
```

## Interface vs Type Alias

- **interface**: Object shapes (extensible, better error messages)
- **type**: Unions, intersections, mapped types

```tsx
// Use interface for object shapes
export interface StaminaCardProps {
  type: string;
  data: StaminaResource;
}

// Use type for unions
export type ResourceData =
  | StaminaResource
  | CooldownResource
  | ExpeditionResource;

// Use type for deriving
import type { ProgressBarProps } from "@/modules/ui/components/ProgressBar";
type ProgressColor = NonNullable<ProgressBarProps["color"]>;
```

## Global Type Augmentation

Place ambient declarations in `src/types/*.d.ts` files:

```tsx
// src/types/intl.d.ts
import type {
  DurationFormat as DurationFormatImpl,
  DurationFormatOptions,
} from "@formatjs/intl-durationformat";

declare global {
  namespace Intl {
    type DurationFormat = DurationFormatImpl;
    const DurationFormat: {
      new (locales?: string | string[], options?: DurationFormatOptions): DurationFormat;
    };
  }
}
```

## Component Props

Extend base library props with style variants:

```tsx
// Bad: Loose props
interface ButtonProps {
  onClick?: () => void;
  children?: React.ReactNode;
}

// Good: Composed props
import type { ButtonProps as AriaButtonProps } from "react-aria-components";
import type { VariantProps } from "tailwind-variants";

export interface ButtonProps extends AriaButtonProps, VariantProps<typeof buttonStyle> {
  className?: string;
}

export const Button: React.FC<ButtonProps> = ({ variant, size, ...props }) => {
  // ...
};
```

## Importing Types

Always use `import type` for type-only imports:

```tsx
// Good: Explicit type imports
import type { GameId } from "@/modules/games/games.types";
import type { StaminaResource } from "@/modules/resources/resources.types";
import type { AppConfig } from "@/modules/settings/settings.types";

// Also good: Mixed import
import { GAME_METADATA, GAME_ORDER } from "@/modules/games/games.constants";
import type { GameId } from "@/modules/games/games.types";
```

## Checklist

- [ ] Types co-located in module `*.types.ts` files
- [ ] Config types use snake_case (match Rust)
- [ ] Resource types use camelCase (serde transformed)
- [ ] Type guards check `unknown` input
- [ ] Literal unions for known string values
- [ ] `Record<UnionType, T>` for exhaustive metadata
- [ ] Interface for objects, type for unions
- [ ] Use `import type` for type-only imports
