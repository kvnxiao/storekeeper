# Storekeeper

Storekeeper is a desktop application that provides real-time stamina resource tracking for various gacha games such as Genshin Impact, Honkai Star Rail, Zenless Zone Zero, Wuthering Waves, etc.

## Before Task Completion

**IMPORTANT**: Before completing any task, ALWAYS run linter auto-fix commands and address any unfixed issues:

```bash
just fix-web  # Lint and apply fixes + formatting for frontend code
just fix      # Lint and apply fixes + formatting rust code
```

Fix all warnings and errors before marking work as done, no exceptions. If there is a large occurrence of the same linter error, provide recommended suggestions via `AskUserQuestion` for the user to pick. DO NOT automatically go and turn the linter rule off unless explicitly stated by the user.

## Tool Usage

- ALWAYS use `pnpm` over `npm`, same applies with `pnpm exec` or `pnpm dlx` over `npx`.

## Quick References

- [DEVELOPMENT.md](DEVELOPMENT.md) — Architecture, setup, conventions, adding features
- [justfile](justfile) — Available project commands
- [Cargo.toml](Cargo.toml) — Workspace dependencies for Rust
- [package.json](frontend/package.json) — Frontend dependencies and pnpm scripts

### Rust Standards

Rust coding standards are in `docs/standards/rust/`:

- [`rust-linting.md`](docs/standards/rust/rust-linting.md) — Clippy configuration with pedantic lints, deny warnings, cast checks
- [`rust-error-handling.md`](docs/standards/rust/rust-error-handling.md) — Handling errors using `thiserror`/`anyhow`, Result propagation with `?`, no `unwrap()`/`expect()`
- [`rust-defensive-programming.md`](docs/standards/rust/rust-defensive-programming.md) — Input validation at boundaries, builder patterns, newtypes for safety, safe indexing, checked arithmetic
- [`rust-code-quality.md`](docs/standards/rust/rust-code-quality.md) — Module organization, enums over booleans, avoid stringly-typed code
- [`rust-api-design.md`](docs/standards/rust/rust-api-design.md) — `#[must_use]` annotations, `impl AsRef/Into` parameter patterns
- [`rust-unsafe.md`](docs/standards/rust/rust-unsafe.md) — Avoiding unsafe code, using safe wrapper crates
- [`rust-testing.md`](docs/standards/rust/rust-testing.md) — Test organization, use `.expect()` with descriptive messages in tests, property-based testing
- [`rust-documentation.md`](docs/standards/rust/rust-documentation.md) — Public API documentation requirements with examples, errors, and arguments sections
- [`rust-performance.md`](docs/standards/rust/rust-performance.md) — Borrowing over cloning, `Cow<str>`, pre-allocation with capacity, avoiding copies
- [`rust-dependencies.md`](docs/standards/rust/rust-dependencies.md) — Version management, minimal feature flags, security auditing
- [`rust-workspaces.md`](docs/standards/rust/rust-workspaces.md) — Multi-crate workspace structure, shared dependencies and lints, avoiding circular dependencies

### Frontend Standards

Frontend is built using React 19, TanStack Start, Vite, TailwindCSS, and Motion. Code is organized into feature-based modules under `src/modules/`. Frontend coding standards are in `docs/standards/frontend/`:

- [`frontend-linting.md`](docs/standards/frontend/frontend-linting.md) — Biome configuration, critical rules, import organization with `@/modules/*` paths
- [`frontend-components.md`](docs/standards/frontend/frontend-components.md) — Module structure, file naming suffixes, import rules
- [`frontend-react-aria.md`](docs/standards/frontend/frontend-react-aria.md) — React Aria integration, props patterns, `composeRenderProps`
- [`frontend-styling.md`](docs/standards/frontend/frontend-styling.md) — Semantic color tokens, tailwind-variant `tv()` patterns, dark mode strategy
- [`frontend-types.md`](docs/standards/frontend/frontend-types.md) — Module-based type organization, type guards, snake_case vs camelCase conventions
- [`frontend-queries.md`](docs/standards/frontend/frontend-queries.md) — TanStack Query options pattern, direct Tauri imports
- [`frontend-atoms.md`](docs/standards/frontend/frontend-atoms.md) — Jotai atoms, effect atoms, action atoms
- [`frontend-forms.md`](docs/standards/frontend/frontend-forms.md) — Form state, dirty tracking, initialization patterns
- [`frontend-routing.md`](docs/standards/frontend/frontend-routing.md) — TanStack Router, view transitions, router-integrated links
- [`frontend-testing.md`](docs/standards/frontend/frontend-testing.md) — Vitest + Testing Library, accessible queries, mocking Tauri
- [`frontend-performance.md`](docs/standards/frontend/frontend-performance.md) — Memoization, animation performance, bundle optimization
- [`frontend-accessibility.md`](docs/standards/frontend/frontend-accessibility.md) — React Aria foundation, touch targets, reduced motion
- [`frontend-dependencies.md`](docs/standards/frontend/frontend-dependencies.md) — pnpm configuration, version strategy, security auditing
- [`frontend-tauri-bridge.md`](docs/standards/frontend/frontend-tauri-bridge.md) — Data exchange conventions between Rust and TypeScript
