# Linter and Formatter Configuration

## Biome

All frontend code uses [Biome](https://biomejs.dev/) for linting and formatting. Configuration lives in `frontend/biome.json`.

### Formatter Settings

```json
{
  "formatter": {
    "indentStyle": "space",
    "indentWidth": 2
  },
  "javascript": {
    "formatter": { "quoteStyle": "double" }
  },
  "css": {
    "formatter": { "quoteStyle": "double" },
    "parser": { "tailwindDirectives": true }
  }
}
```

### Domain Configuration

Enable all rules for React and project domains:

```json
{
  "linter": {
    "domains": {
      "project": "all",
      "react": "all"
    }
  }
}
```

## Critical Rules

These rules are set to `error` and must never be disabled:

| Rule | Purpose |
|------|---------|
| `useHookAtTopLevel` | Prevents conditional hook calls |
| `noFloatingPromises` | Forces explicit promise handling |
| `noImportCycles` | Prevents circular dependencies |
| `noMisusedPromises` | Catches promises in wrong contexts |
| `useExhaustiveSwitchCases` | Ensures all union cases handled |

### Promise Handling

```tsx
// Bad: Floating promise
fetchData();

// Good: Explicit handling
await fetchData();
// or
void fetchData(); // Explicit discard
```

### Hook Placement

```tsx
// Bad: Conditional hook
if (condition) {
  const [state, setState] = useState();
}

// Good: Top-level hook
const [state, setState] = useState();
if (condition) {
  // use state
}
```

## Disabling Rules

Use file-level overrides in `biome.json` for framework requirements:

```json
{
  "overrides": [{
    "includes": ["src/routes/__root.tsx"],
    "linter": {
      "rules": { "style": { "noHeadElement": "off" } }
    }
  }]
}
```

For inline disables, always include justification:

```tsx
// biome-ignore lint/style/noHeadElement: Required for TanStack Start root
<head>...</head>
```

**Never disable**: `noFloatingPromises`, `noMisusedPromises`, `useHookAtTopLevel`, `noImportCycles`

## Import Organization

Biome auto-organizes imports on save. Follow this order:

```tsx
// 1. External packages
import { useAtomValue } from "jotai";
import { useMemo } from "react";

// 2. Path aliases (@/modules/*)
import { currentTick } from "@/modules/core/core.tick";
import { ProgressBar } from "@/modules/ui/components/ProgressBar";
import { resourcesQueryAtom } from "@/modules/resources/resources.atoms";
import type { StaminaResource } from "@/modules/resources/resources.types";
import { formatTimeRemaining } from "@/modules/resources/resources.utils";
```

**Note**: No relative imports allowed - always use `@/modules/*` paths.

## Commands

```bash
just lint-web  # Check for issues
just fix-web   # Auto-fix and format
pnpm lint      # Direct Biome lint
pnpm fix       # Direct Biome fix
```

**Always run `just fix-web` before committing.**
