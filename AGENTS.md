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

- [DEVELOPMENT.md](DEVELOPMENT.md) — Quick-start: prerequisites, commands, configuration
- [justfile](justfile) — Available project commands
- [Cargo.toml](Cargo.toml) — Workspace dependencies for Rust
- [package.json](frontend/package.json) — Frontend dependencies and pnpm scripts

Read [`docs/README.md`](docs/README.md) for the full documentation index.

- Consult `docs/architecture/` for system design and data flow
- Consult `docs/onboarding/` for setup and contribution guides
- Follow all patterns in `docs/standards/rust/`
- Follow all patterns in `docs/standards/frontend/`
