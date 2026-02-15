# File Naming Conventions

TypeScript files use a `<module>.<type>.ts` naming pattern where `<type>` indicates the file's role.

## Extension Types

| Suffix | Purpose | Example |
|---|---|---|
| `.atoms.ts` | Jotai atom container classes | `core.atoms.ts`, `genshin.atoms.ts` |
| `.query.ts` | TanStack Query options and mutations | `resources.query.ts`, `settings.query.ts` |
| `.types.ts` | TypeScript type/interface definitions | `resources.types.ts`, `settings.types.ts` |
| `.hooks.ts` | Custom React hooks | `resources.hooks.ts` |
| `.utils.ts` | Pure utility/helper functions | `resources.utils.ts` |
| `.constants.ts` | Static constant values | `games.constants.ts` |
| `.styles.ts` | Style definitions (Tailwind helpers, etc.) | `ui.styles.ts` |
| `.animations.ts` | Animation definitions | `ui.animations.ts` |
| `.config.ts` | Configuration setup (query clients, etc.) | `core.config.ts`, `core.queryClient.ts` |

## Rules

- The `<module>` prefix matches the parent directory name (e.g., `resources/resources.query.ts`)
- One file per role — don't mix atoms and query options in the same file
- React components use PascalCase `.tsx` files without the `.<type>` suffix (e.g., `StaminaCard.tsx`)
- Auto-generated files (e.g., `routeTree.gen.ts`) are excluded from this convention

## Checklist

- [ ] File uses `<module>.<type>.ts` pattern
- [ ] `<module>` prefix matches parent directory
- [ ] Single responsibility per file (no mixing types)
