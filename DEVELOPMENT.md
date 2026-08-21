# Development Guide

Storekeeper is a cross-platform desktop application built with [Tauri](https://tauri.app/) that tracks stamina resources for gacha games. The backend is a Rust workspace; the frontend is SolidJS with TanStack Start/Router/Query/Form.

## Prerequisites

| Tool | Details |
|------|---------|
| [Rust 1.95+](https://rustup.rs/) | 2024 edition |
| Rust nightly toolchain | Used by `cargo +nightly fmt` for formatting only |
| [fnm](https://github.com/Schniz/fnm) | See `frontend/.node-version` for Node.js version |
| [pnpm](https://pnpm.io/) | See `frontend/package.json` `packageManager` field for version |
| [Vite+](https://viteplus.dev/) | `vp` CLI: frontend checks, fixes, tests, and dependency updates |
| [just](https://github.com/casey/just) | Command runner |
| [tauri-cli](https://v2.tauri.app/reference/cli/) | Tauri commands |
| [Platform deps](https://v2.tauri.app/start/prerequisites/) | OS-specific Tauri build dependencies |

## Commands

```bash
just dev      # Run tauri desktop app in dev mode
just lint     # Run clippy and check formatting
just fix      # Lint and apply fixes + formatting
just test     # Run Rust tests
just audit    # Check dependency advisories, licenses, and sources
just lint-web # Lint, format check, and type check frontend code
just fix-web  # Lint and apply fixes + formatting for frontend code
just test-web # Run frontend tests
just bundle   # Create tauri release bundle
```

See the [justfile](justfile) for all available commands.

Never disable a lint rule to silence an error. If the same rule fires repeatedly, stop and ask how to proceed.

### Frontend

Run from the `frontend/` directory:

```bash
pnpm install  # Install dependencies, then compile i18n messages (postinstall)

# NOTE: dev server and production builds should use `just dev` and `just bundle`
```

Use `vp` (backed by pnpm) for frontend checks, fixes, tests, and dependency updates. Use `pnpm install` to install dependencies and `pnpm` or `pnpm exec` when `vp` does not provide the command. Do not use `npm` or `npx`.

## Localization

`locales/` holds one catalog per supported locale, shared by the backend (tray labels, OS notifications) and the frontend (all UI text). Adding a string means adding one key to each locale file. Compiled message output for the frontend is generated and gitignored; `pnpm install` regenerates it.

## Configuration

Configuration files are loaded from:

| Platform | Config Directory |
|----------|----------------------------------------------|
| Windows  | `%APPDATA%\storekeeper\` |
| macOS    | `~/Library/Application Support/storekeeper/` |
| Linux    | `~/.config/storekeeper/` |

Config files (`config.toml` and `secrets.toml`) are auto-created with commented templates on first `just dev` run.

## Logging

Log lines go to stdout in a human-readable format and to `logs/` under the config
directory as one JSON object per line, rotated daily with seven files retained.
Settings offers two ways into the file: **View Logs** opens the in-app viewer on
the current day's file, and **Open Log Folder** reveals the directory.

The **Log Level** dropdown applies without a restart. When `RUST_LOG` is set it
overrides the dropdown for the whole process, so a developer override survives a
settings save.

## Continuous Integration

GitHub Actions runs Rust linting, Rust tests, and frontend linting for pull requests and pushes to `main` (`.github/workflows/`). Run `just lint`, `just test`, and `just lint-web` locally for the checks CI runs. Run `just test-web` for frontend tests.

## Further Reading

See [`docs/`](docs/README.md) for architecture and onboarding documentation:

- **[Architecture](docs/architecture/)** for system design, crate layout, data flow
- **[Onboarding](docs/onboarding/)** for environment setup and adding a new game
- **[Standards](.claude/skills/)** for Rust and frontend coding conventions (`rust-rules`, `solidjs-rules`). The repository also stores these skills under `.agents/skills/`.
