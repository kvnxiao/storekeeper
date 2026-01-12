# Development Guide

## Project Overview

Storekeeper is a cross-platform desktop application built with [Tauri](https://tauri.app/) that tracks stamina resources for gacha games. The backend is written in Rust as a multi-crate workspace, and the frontend uses React 19 with TanStack Start, TypeScript, and Tailwind CSS.

## Architecture

```text
storekeeper/
├── storekeeper-core/           # Core traits, types, config, errors
├── storekeeper-client-core/    # Shared HTTP client with retry/middleware
├── storekeeper-client-hoyolab/ # HoYoLab API client (Genshin, HSR, ZZZ)
├── storekeeper-client-kuro/    # Kuro Games API client (Wuthering Waves)
├── storekeeper-game-genshin/   # Genshin Impact GameClient implementation
├── storekeeper-game-hsr/       # Honkai: Star Rail GameClient implementation
├── storekeeper-game-zzz/       # Zenless Zone Zero GameClient implementation
├── storekeeper-game-wuwa/      # Wuthering Waves GameClient implementation
├── storekeeper-app-tauri/      # Tauri app: UI, tray, notifications, polling
└── frontend/                   # React + TanStack Start + TypeScript + Tailwind + Vite
```

### Crate Responsibilities

- **storekeeper-core**: Foundation layer defining `GameClient` and `DailyRewardClient` traits, resource types (`StaminaResource`, `CooldownResource`), region/game ID enums, configuration parsing, and shared error types.
- **storekeeper-client-core**: HTTP infrastructure with reqwest, retry policies, rate limiting, and middleware. Used by all API client crates.
- **storekeeper-client-hoyolab**: HoYoLab API authentication (cookie-based), dynamic secret (DS) generation for request signing, and API request helpers for miHoYo games.
- **storekeeper-client-kuro**: Kuro Games API client with automatic credential loading from the launcher cache file.
- **storekeeper-game-\***: Per-game crates implementing `GameClient`. Each defines game-specific resource types (e.g., Resin, Trailblaze Power) and API response parsing.
- **storekeeper-app-tauri**: Application entry point. Orchestrates game clients, runs the polling loop, manages system tray, and sends desktop notifications.
- **frontend/**: Web UI rendered in the Tauri webview. Communicates with the Rust backend via Tauri's IPC.

## Prerequisites

- [Rust 1.85+](https://rustup.rs/) (2024 edition)
- [fnm](https://github.com/Schniz/fnm) for Node.js version management (see `frontend/.node-version` for version)
- [pnpm](https://pnpm.io/) (see `frontend/package.json` for version)
- [just](https://github.com/casey/just) command runner
- [tauri-cli](https://v2.tauri.app/reference/cli/) to run tauri commands
- [Platform-specific Tauri dependencies](https://v2.tauri.app/start/prerequisites/) to ensure the desktop UI builds correctly

## Commands

```bash
just dev      # Run tauri desktop app in dev mode
just lint     # Run clippy and check formatting
just fix      # Lint and apply fixes + formatting
just lint-web # Lint frontend code
just fix-web   # Lint and apply fixes + formatting for frontend code
just test     # Run tests
just bundle   # Create tauri release bundle
```

See the [justfile](justfile) for all available commands.

### Frontend Commands

Run from the `frontend/` directory:

```bash
pnpm install  # Install dependencies

# NOTE: running the dev server and building for production should be delegated to the `just dev` and `just bundle` commands above
```

## Configuration

Configuration files are loaded from:

| Platform | Config Directory                             |
| -------- | -------------------------------------------- |
| Windows  | `%APPDATA%\storekeeper\`                     |
| macOS    | `~/Library/Application Support/storekeeper/` |
| Linux    | `~/.config/storekeeper/`                     |

Copy the example files to the appropriate config directory to get started:

- `config.example.toml` → `config.toml`
- `secrets.example.toml` → `secrets.toml`

## Conventions

For detailed coding standards, see [docs/standards/](docs/standards/):
- [Rust standards](docs/standards/rust/) — Linting, error handling, testing, performance
- [Frontend standards](docs/standards/frontend/) — Components, state management, styling

### Async Runtime

The application uses Tauri's built-in async runtime (`tauri::async_runtime`), which is backed by Tokio with the following features enabled:

- `rt-multi-thread` - Multi-threaded runtime
- `sync` - Synchronization primitives (`RwLock`, channels)
- `time` - Timer utilities for scheduled tasks
- `signal` - Signal handling (Ctrl+C)

**Spawning tasks**: Use `tauri::async_runtime::spawn()` for background tasks:

```rust
tauri::async_runtime::spawn(async move {
    // Background work
});
```

**Cancellation**: Use `tokio_util::sync::CancellationToken` for graceful shutdown. Long-running loops should use `tokio::select!` to respond to cancellation:

```rust
loop {
    tokio::select! {
        () = cancel_token.cancelled() => break,
        () = tokio::time::sleep(interval) => {
            // Periodic work
        }
    }
}
```

### State Management

Tauri's application state uses `Arc<RwLock<StateData>>` for thread-safe concurrent access:

```rust
pub struct AppState {
    pub inner: Arc<RwLock<StateData>>,
}
```

**Registration**: State is registered via `app.manage()` during setup.

**Access patterns**:

- In Tauri commands: `state: State<'_, AppState>`
- In background tasks: `app_handle.state::<AppState>()`
- Optional access: `app_handle.try_state::<T>()`

**Async methods**: All state access is async using `.read().await` and `.write().await`.

**Frontend communication**: Use `app_handle.emit(EVENT_NAME, &data)` to send events to the frontend.

**Rate limiting**: When fetching from multiple game clients, requests are executed sequentially within each API provider (to avoid rate limits) but in parallel across different providers.

### Tauri-Frontend Bridge

Data exchange between Rust and TypeScript uses specific naming conventions:

- **Config/Secrets (Tauri ↔ Frontend)**: snake_case (`poll_interval_secs`)
- **Resource types**: camelCase (`fullAt`, `regenRateSeconds`)
- **GameId enum**: SCREAMING_SNAKE_CASE (`GENSHIN_IMPACT`)

Config/secrets types flow directly without DTO conversion. See [`frontend-tauri-bridge.md`](/docs/standards/frontend/frontend-tauri-bridge.md) for details.

### Adding a New Game

1. Create a new crate: `storekeeper-game-<name>/`
2. Define resource types implementing `StaminaResource` or other core traits
3. Implement `GameClient` trait
4. If using a new API provider, create a client crate (`storekeeper-client-<provider>/`)
5. Add the game to `storekeeper-app-tauri` dependencies and wire it up
6. Add configuration options to `config.example.toml`

### Workspace Structure

Crates are placed at the root level (not in a `crates/` subdirectory) for simpler navigation. See the workspace `Cargo.toml` for shared dependencies and lint configuration.
