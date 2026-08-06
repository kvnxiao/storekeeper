# Development Guide

Storekeeper is a cross-platform desktop application built with [Tauri](https://tauri.app/) that tracks stamina resources for gacha games. The backend is a Rust workspace; the frontend is SolidJS with TanStack Start/Router/Query/Form.

## Prerequisites

| Tool | Details |
|------|---------|
| [Rust 1.95+](https://rustup.rs/) | 2024 edition |
| Rust nightly toolchain | Used by `cargo +nightly fmt` for formatting only |
| [fnm](https://github.com/Schniz/fnm) | See `frontend/.node-version` for Node.js version |
| [pnpm](https://pnpm.io/) | See `frontend/package.json` `packageManager` field for version |
| [Vite+](https://viteplus.dev/) | `vp` CLI: frontend dev server, build, lint, format, test |
| [just](https://github.com/casey/just) | Command runner |
| [tauri-cli](https://v2.tauri.app/reference/cli/) | Tauri commands |
| [Platform deps](https://v2.tauri.app/start/prerequisites/) | OS-specific Tauri build dependencies |

## Commands

```bash
just dev      # Run tauri desktop app in dev mode
just lint     # Run clippy and check formatting
just fix      # Lint and apply fixes + formatting
just test     # Run Rust tests
just lint-web # Lint, format check, and type check frontend code
just fix-web  # Lint and apply fixes + formatting for frontend code
just test-web # Run frontend tests
just bundle   # Create tauri release bundle
```

See the [justfile](justfile) for all available commands.

Never disable a lint rule to silence an error. If a rule fires widely, raise it rather than suppressing it.

### Frontend

Run from the `frontend/` directory:

```bash
pnpm install  # Install dependencies, then compile i18n messages (postinstall)

# NOTE: dev server and production builds should use `just dev` and `just bundle`
```

Frontend tooling goes through `vp` (backed by pnpm) for dependencies, checks, and tests, and `pnpm`/`pnpm exec` otherwise. Do not use `npm` or `npx`.

## Localization

`locales/` holds one catalog per supported locale, shared by the backend (tray labels, OS notifications) and the frontend (all UI text). Adding a string means adding one key to each locale file. The frontend's compiled message output is generated and gitignored; `pnpm install` regenerates it.

## Configuration

Configuration files are loaded from:

| Platform | Config Directory |
|----------|----------------------------------------------|
| Windows  | `%APPDATA%\storekeeper\` |
| macOS    | `~/Library/Application Support/storekeeper/` |
| Linux    | `~/.config/storekeeper/` |

Config files (`config.toml` and `secrets.toml`) are auto-created with commented templates on first `just dev` run.

## Continuous Integration

GitHub Actions runs Rust linting, Rust tests, and frontend linting on pull requests (`.github/workflows/`). Running `just fix`, `just test`, `just fix-web`, and `just test-web` locally covers the same ground.

## Further Reading

See [`docs/`](docs/README.md) for comprehensive documentation:

- **[Architecture](docs/architecture/)** for system design, crate layout, data flow
- **[Onboarding](docs/onboarding/)** for environment setup and adding a new game
- **[Standards](.claude/skills/)** for Rust and frontend coding conventions (`rust-rules`, `solidjs-rules`)
