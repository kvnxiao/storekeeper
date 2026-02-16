# Development Guide

Storekeeper is a cross-platform desktop application built with [Tauri](https://tauri.app/) that tracks stamina resources for gacha games.

## Prerequisites

| Tool | Details |
|------|---------|
| [Rust 1.85+](https://rustup.rs/) | 2024 edition |
| [fnm](https://github.com/Schniz/fnm) | See `frontend/.node-version` for Node.js version |
| [pnpm](https://pnpm.io/) | See `frontend/package.json` `packageManager` field for version |
| [just](https://github.com/casey/just) | Command runner |
| [tauri-cli](https://v2.tauri.app/reference/cli/) | Tauri commands |
| [Platform deps](https://v2.tauri.app/start/prerequisites/) | OS-specific Tauri build dependencies |

## Commands

```bash
just dev      # Run tauri desktop app in dev mode
just lint     # Run clippy and check formatting
just fix      # Lint and apply fixes + formatting
just lint-web # Lint frontend code
just fix-web  # Lint and apply fixes + formatting for frontend code
just test     # Run tests
just bundle   # Create tauri release bundle
```

See the [justfile](justfile) for all available commands.

### Frontend

Run from the `frontend/` directory:

```bash
pnpm install  # Install dependencies

# NOTE: dev server and production builds should use `just dev` and `just bundle`
```

## Configuration

Configuration files are loaded from:

| Platform | Config Directory |
|----------|----------------------------------------------|
| Windows  | `%APPDATA%\storekeeper\` |
| macOS    | `~/Library/Application Support/storekeeper/` |
| Linux    | `~/.config/storekeeper/` |

Config files (`config.toml` and `secrets.toml`) are auto-created with commented templates on first `just dev` run.

## Further Reading

See [`docs/`](docs/README.md) for comprehensive documentation:

- **[Architecture](docs/architecture/)** — System design, crate layout, data flow
- **[Onboarding](docs/onboarding/)** — Environment setup, adding a new game
- **[Standards](docs/standards/)** — Rust and frontend coding conventions
