# Development Setup

This guide walks through setting up a local development environment for Storekeeper.

## Prerequisites

Install the following tools:

| Tool | Version | Purpose |
|------|---------|---------|
| [Rust](https://rustup.rs/) | 1.85+ (2024 edition) | Backend compilation |
| [fnm](https://github.com/Schniz/fnm) | Latest | Node.js version management |
| [pnpm](https://pnpm.io/) | See `frontend/package.json` `packageManager` | Frontend package manager |
| [just](https://github.com/casey/just) | Latest | Command runner |
| [tauri-cli](https://v2.tauri.app/reference/cli/) | v2 | Desktop app bundling |
| [Platform deps](https://v2.tauri.app/start/prerequisites/) | — | OS-specific build tools |

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

Storekeeper requires two config files in the platform-specific config directory:

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\storekeeper\` |
| macOS | `~/Library/Application Support/storekeeper/` |
| Linux | `~/.config/storekeeper/` |

Copy the example files:

```bash
# Adjust the target path for your OS
cp config.example.toml <CONFIG_DIR>/config.toml
cp secrets.example.toml <CONFIG_DIR>/secrets.toml
```

Edit `config.toml` to enable the games you play and `secrets.toml` to add your credentials. See [README.md](../../README.md#configuration-setup) for detailed credential instructions.

## Running

```bash
just dev      # Start Tauri dev server (backend + frontend hot reload)
```

This compiles the Rust backend and starts the Vite dev server for the frontend. Changes to either side trigger hot reload.

## Common Commands

```bash
just lint      # Run clippy + format check (Rust)
just fix       # Auto-fix lint issues + format (Rust)
just lint-web  # Run Biome lint + TypeScript check (frontend)
just fix-web   # Auto-fix lint + format (frontend)
just test      # Run Rust tests
just bundle    # Create release build
```

See the [justfile](../../justfile) for all available commands.

## Project Structure

The codebase is a Rust workspace with 9 crates plus a React frontend. See [02-directory-structure.md](../architecture/02-directory-structure.md) for the full layout and dependency graph.

Key entry points:
- **Rust**: `storekeeper-app-tauri/src/lib.rs` — Application setup and lifecycle
- **Frontend**: `frontend/src/routes/index.tsx` — Dashboard page

## Coding Standards

Before submitting code, familiarise yourself with the project standards:
- [Rust standards](../standards/rust/) — Linting, error handling, testing, performance
- [Frontend standards](../standards/frontend/) — Components, state management, styling

**Always run linters before committing**:

```bash
just fix       # Rust
just fix-web   # Frontend
```
