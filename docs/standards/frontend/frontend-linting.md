# Linter and Formatter Configuration

## Vite+

All frontend tooling runs through [Vite+](https://viteplus.dev/) (the `vp` CLI), a
unified toolchain that bundles Vite, Vitest, Oxlint (lint), Oxfmt (format), and
type-aware checking via tsgolint. Linter and formatter configuration lives in the
`lint` and `fmt` blocks of `frontend/vite.config.ts` — there is no standalone lint
or format config file.

```ts
// frontend/vite.config.ts
export default defineConfig({
  // ...vite plugins, server, etc.
  lint: {
    plugins: ["react", "import", "typescript", "unicorn", "jsx-a11y"],
    categories: { correctness: "error", suspicious: "error" },
    options: { typeAware: true, typeCheck: true },
    rules: { /* see Critical Rules below */ },
    ignorePatterns: ["src/routeTree.gen.ts", "src/paraglide/**", /* ... */],
  },
  fmt: {
    // Oxfmt is Prettier-compatible; defaults are 2-space indent, double quotes.
    ignorePatterns: ["src/routeTree.gen.ts", "src/paraglide/**", /* ... */],
  },
});
```

### Type-Aware Linting

`options.typeAware` enables rules that require TypeScript type information (powered
by tsgolint), and `options.typeCheck` runs full type-checking as part of `vp lint` /
`vp check` — so a separate `tsc --noEmit` is no longer needed.

### Formatter Settings

Oxfmt formats JS/TS/JSON and CSS with 2-space indentation and double quotes (its
Prettier-compatible defaults, matching the previous configuration).

## Critical Rules

These rules are set to `error` and must never be disabled (oxlint names):

| Rule | Purpose |
|------|---------|
| `react/rules-of-hooks` | Prevents conditional hook calls |
| `typescript/no-floating-promises` | Forces explicit promise handling |
| `import/no-cycle` | Prevents circular dependencies |
| `typescript/no-misused-promises` | Catches promises in wrong contexts |
| `typescript/switch-exhaustiveness-check` | Ensures all union cases handled |

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

For framework requirements that apply to specific files, use `lint.overrides` in
`vite.config.ts`:

```ts
lint: {
  overrides: [
    { files: ["src/routes/__root.tsx"], rules: { "some/rule": "off" } },
  ],
}
```

For inline disables, use oxlint directives and always include justification:

```tsx
// oxlint-disable-next-line jsx-a11y/prefer-tag-over-role -- Badge is a styled span
role="button"
```

**Never disable**: `typescript/no-floating-promises`, `typescript/no-misused-promises`,
`react/rules-of-hooks`, `import/no-cycle`.

## Import Organization

Follow this order (maintained by convention — Oxfmt does not reorder imports):

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
just lint-web   # Check: format + lint + type-check
just fix-web    # Auto-fix lint + apply formatting
vp check        # Direct: format + lint + type-check
vp check --fix  # Direct: auto-fix lint + formatting
vp fmt          # Format only
vp lint         # Lint only (+ type-check)
```

**Always run `just fix-web` before committing.**
