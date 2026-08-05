# Storekeeper

Desktop app (Tauri + Rust backend, SolidJS frontend) for real-time stamina
tracking across gacha games (Genshin Impact, Honkai Star Rail, Zenless Zone
Zero, Wuthering Waves, ...).

## Commands

All tasks run through `just` (see `justfile` for the full list):

- `just fix` / `just fix-web` - lint auto-fix + format (Rust / frontend).
  Run the relevant one before finishing any change; fix remaining warnings.
- `just test` / `just test-web` - Rust / frontend tests
- `just dev` - run the app

Never disable a lint rule to silence errors. If one rule fires many times,
stop and ask how to proceed.

## Conventions

- Frontend tooling is `vp` (vite-plus) backed by `pnpm`: use `vp` commands for
  deps, checks, and tests, and `pnpm` (`pnpm exec`, `pnpm dlx`) otherwise -
  never `npm`/`npx`.
- Always invoke the matching `*-rules` skill before touching code in its area:
  `solidjs-rules` (frontend), `rust-rules` (Rust), `github-actions-rules`
  (workflows). Agents without skill support read them from `.claude/skills/`.

## Docs

- `DEVELOPMENT.md` - prerequisites, commands, configuration
- `docs/README.md` - index for `docs/architecture/` and `docs/onboarding/`
