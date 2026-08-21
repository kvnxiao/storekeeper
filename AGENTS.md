# Storekeeper

Desktop app (Tauri + Rust backend, SolidJS frontend) for real-time stamina
tracking across gacha games (Genshin Impact, Honkai Star Rail, Zenless Zone
Zero, Wuthering Waves, ...).

## Commands

Use `just` for repository tasks; see `justfile` for the full list.

- For Rust changes, run `just fix`; for frontend changes, run `just fix-web`.
  Fix remaining warnings before finishing.
- Run `just test` for Rust changes and `just test-web` for frontend changes.
- Run `just dev` to start the app.

Never disable a lint rule to silence an error. If the same rule fires
repeatedly, stop and ask how to proceed.

## Development conventions

- Use `vp` for frontend dependency, check, and test commands. Use `pnpm`
  (`pnpm exec`, `pnpm dlx`) when `vp` does not provide the command; never use
  `npm` or `npx`.
- Add a sibling test file for every frontend `*.utils.ts` and `*.state.ts` file.
  Keep derivation logic in pure functions so it is testable without rendering.
- Before touching frontend, Rust, or workflow code, use the matching skill:
  `solidjs-rules` (frontend), `rust-rules` (Rust), or `github-actions-rules`
  (workflows). When the skill is unavailable, read it from `.agents/skills/`
  or `.claude/skills/`.

## Managed content

- Treat `.agents/` and `.claude/` as curated or third-party-installed content.
- Do not run completion-check review, simplification, or prose-correction passes
  over files under these directories.
- Preserve those files verbatim unless the task explicitly targets them.

## Docs

- `DEVELOPMENT.md` - prerequisites, commands, configuration
- `docs/README.md` - index for `docs/architecture/` and `docs/onboarding/`
