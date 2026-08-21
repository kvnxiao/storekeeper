# Development Setup

This guide walks through setting up a local development environment for Storekeeper.

## Prerequisites

Install the following tools:

| Tool | Version | Purpose |
|------|---------|---------|
| [Rust](https://rustup.rs/) | 1.95+ (2024 edition) | Backend compilation |
| Rust nightly toolchain | Latest | `cargo +nightly fmt` (formatting only) |
| [fnm](https://github.com/Schniz/fnm) | Latest | Node.js version management |
| [pnpm](https://pnpm.io/) | See `frontend/package.json` `packageManager` | Frontend package manager |
| [Vite+](https://viteplus.dev/) | Latest (`vp` CLI) | Frontend checks, fixes, tests, and dependency updates |
| [just](https://github.com/casey/just) | Latest | Command runner |
| [tauri-cli](https://v2.tauri.app/reference/cli/) | v2 | Desktop app bundling |
| [Platform deps](https://v2.tauri.app/start/prerequisites/) | n/a | OS-specific build tools |

## Clone and Install

```bash
git clone https://github.com/kvnxiao/storekeeper.git
cd storekeeper

# Install frontend dependencies
cd frontend
fnm use          # Use the Node.js version from .node-version
pnpm install
cd ..
```

## Configuration

Storekeeper auto-creates `config.toml` and `secrets.toml` with commented templates on first launch. No manual setup is needed to get started.

Config files are stored in the platform-specific config directory:

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\storekeeper\` |
| macOS | `~/Library/Application Support/storekeeper/` |
| Linux | `~/.config/storekeeper/` |

Edit these files to enable games and add credentials, or use the in-app settings UI. See [README.md](../../README.md#configuration) for credential instructions.

## Running

```bash
just dev      # Start Tauri dev server (backend + frontend hot reload)
```

This compiles the Rust backend and starts the Vite dev server for the frontend. Changes to either side trigger hot reload.

## Common Commands

```bash
just lint      # Run clippy + format check (Rust)
just fix       # Auto-fix lint issues + format (Rust)
just lint-web  # Run format, lint + type checks (frontend, via Vite+)
just fix-web   # Auto-fix lint + format (frontend, via Vite+)
just test      # Run Rust tests
just test-web  # Run frontend tests
just bundle    # Create release build
```

See the [justfile](../../justfile) for all available commands.

## Project Structure

A Rust workspace of nine crates plus a SolidJS frontend. See [02-directory-structure.md](../architecture/02-directory-structure.md) for the layout and dependency graph, and [01-overview.md](../architecture/01-overview.md) for how the layers fit together.

## Coding Standards

Before touching code, read the matching skill from `.agents/skills/` or
`.claude/skills/`:

- [Rust rules](../../.claude/skills/rust-rules/) for linting, error handling, testing, performance
- [SolidJS rules](../../.claude/skills/solidjs-rules/) for components, state architecture, data fetching, i18n

**Always run linters before committing**:

```bash
just fix       # Rust
just fix-web   # Frontend
```
