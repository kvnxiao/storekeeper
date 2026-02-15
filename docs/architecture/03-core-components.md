# Core Components

In-depth analysis of key architectural components and design patterns.

## 1. Trait-Based Abstraction

### GameClient Trait

Defines the contract for all game implementations. Located in `storekeeper-core/src/game.rs`.

```rust
#[async_trait]
pub trait GameClient: Send + Sync {
    type Resource: Send + Serialize;
    type Error: std::error::Error + Send + Sync + 'static;

    fn game_id(&self) -> GameId;
    fn game_name(&self) -> &'static str;
    async fn fetch_resources(&self) -> Result<Vec<Self::Resource>, Self::Error>;
    async fn is_authenticated(&self) -> Result<bool, Self::Error>;
}
```

**Why associated types?** Each game has exactly one resource type — associated types make this 1:1 relationship explicit and avoid generic parameter proliferation.

**Why `Send + Sync`?** Game clients are stored in `AppState` behind `Arc<RwLock<_>>` and accessed from multiple tokio tasks (polling loop, IPC commands, scheduled claims).

### DynGameClient (Type Erasure)

Different `GameClient` implementations have incompatible associated types, so they can't be stored in a single collection. `DynGameClient` solves this by serializing resources to `serde_json::Value`:

```rust
#[async_trait]
pub trait DynGameClient: Send + Sync {
    fn game_id(&self) -> GameId;
    async fn fetch_resources_json(&self)
        -> Result<serde_json::Value, Box<dyn Error + Send + Sync>>;
}

// Blanket implementation: any GameClient is automatically a DynGameClient
#[async_trait]
impl<T: GameClient> DynGameClient for T {
    fn game_id(&self) -> GameId { GameClient::game_id(self) }

    async fn fetch_resources_json(&self)
        -> Result<serde_json::Value, Box<dyn Error + Send + Sync>>
    {
        let resources = self.fetch_resources().await?;
        Ok(serde_json::to_value(resources)?)
    }
}
```

This enables `HashMap<GameId, Box<dyn DynGameClient>>` in the registry.

**Trade-off**: Slight runtime overhead (vtable dispatch + JSON serialization) in exchange for plugin-style extensibility.

### DailyRewardClient Trait

Separate trait for daily reward claiming, located in `storekeeper-core/src/daily_reward.rs`. Separated from `GameClient` because:
- Not all games support daily rewards (Wuthering Waves doesn't)
- Different lifecycle: claim once per day vs poll every N minutes

Uses the same type erasure pattern (`DynDailyRewardClient`) with a blanket implementation.

## 2. Resource Type System

### Core Resource Types

Located in `storekeeper-core/src/resource.rs`. Shared across all games:

| Type | Used For | Examples |
|------|----------|---------|
| `StaminaResource` | Regenerating resources | Resin, Trailblaze Power, Battery, Waveplates |
| `CooldownResource` | One-time cooldowns | Parametric Transformer |
| `ExpeditionResource` | Timed dispatches | Genshin Expeditions |

All use `#[serde(rename_all = "camelCase")]` to convert Rust's snake_case to JavaScript's camelCase at the serialization boundary.

### Game-Specific Resource Enums

Each game wraps core types in a tagged enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum GenshinResource {
    Resin(StaminaResource),
    RealmCurrency(StaminaResource),
    ParametricTransformer(CooldownResource),
    Expeditions(ExpeditionResource),
}
```

Produces JSON discriminated unions:

```json
{ "type": "resin", "data": { "current": 100, "max": 160, "fullAt": "...", "regenRateSeconds": 480 } }
```

The frontend consumes these as TypeScript discriminated unions with `switch (resource.type)`.

## 3. HTTP Client Infrastructure

### HttpClientBuilder

Located in `storekeeper-client-core/src/client.rs`. Uses the builder pattern:

```rust
let client = HttpClientBuilder::new()
    .header_static("x-rpc-app_version", "1.5.0")
    .header_static("x-rpc-client_type", "5")
    .build()?;
```

Two build methods:
- `build()` → plain `reqwest::Client`
- `build_with_retry(max_retries)` → `ClientWithMiddleware` wrapping reqwest with exponential backoff, jitter, and transient error retry

### HoYoLab Authentication

`HoyolabClient` adds two authentication mechanisms per request:
1. **Cookie header**: `ltuid_v2` and `ltoken_v2` from user config
2. **DS header**: Cryptographic signature using MD5 — `md5(salt={salt}&t={timestamp}&r={random})`

### Kuro Authentication

`KuroClient` auto-loads credentials from the Kuro launcher cache file at a known path, requiring no manual credential entry from users.

## 4. Game Client Registry

Located in `storekeeper-app-tauri/src/registry.rs`. Stores type-erased game clients and orchestrates fetching.

```rust
pub struct GameClientRegistry {
    clients: HashMap<GameId, Box<dyn DynGameClient>>,
}
```

### Fetch Strategy

```
fetch_all()
  ├── Group clients by ApiProvider
  ├── Fetch providers in PARALLEL (join_all)
  │    └── Within provider: fetch games SEQUENTIALLY
  │         └── Emit "game-resource-updated" per game (incremental UI updates)
  └── Collect results into HashMap<GameId, Value>
```

**Why sequential within provider?** HoYoLab games (Genshin, HSR, ZZZ) share an API rate limit. Fetching them sequentially avoids 429 errors. Different providers (HoYoLab vs Kuro) have independent rate limits, so they run in parallel.

**Why per-game events?** Emitting `game-resource-updated` after each game completes allows the frontend to progressively render results rather than waiting for all games to finish.

## 5. Application State

Located in `storekeeper-app-tauri/src/state.rs`.

```rust
pub struct AppState {
    pub inner: Arc<RwLock<StateData>>,
}

pub struct StateData {
    pub resources: AllResources,
    pub refreshing: bool,
    pub registry: GameClientRegistry,
    pub daily_reward_registry: DailyRewardRegistry,
    pub daily_reward_status: AllDailyRewardStatus,
    pub config: AppConfig,
}
```

**Concurrency model**: `Arc<RwLock<_>>` allows multiple concurrent readers (Tauri commands) or a single writer (polling updates). All access is async (`.read().await`, `.write().await`).

**Access patterns**:
- Tauri commands: `state: State<'_, AppState>`
- Background tasks: `app_handle.state::<AppState>()`

**Config reload**: `reload_config()` re-reads TOML files and recreates registries without restarting the app.

## 6. Background Tasks

### Polling Loop

Located in `storekeeper-app-tauri/src/polling.rs`. Uses `tokio::select!` with a `CancellationToken` for graceful shutdown:

```rust
loop {
    tokio::select! {
        () = cancel_token.cancelled() => break,
        () = tokio::time::sleep(poll_interval) => {
            poll_resources(&app_handle).await;
        }
    }
}
```

A `refreshing` flag in state prevents overlapping fetches.

### Scheduled Claims

Located in `storekeeper-app-tauri/src/scheduled_claim.rs`. Runs on a separate tokio task:
1. **Startup**: Checks and claims any unclaimed rewards
2. **Scheduled loop**: Calculates next claim time, sleeps until then, claims with retry and exponential backoff

Retries on transient errors (timeout, connection, network, DNS, reset) with configurable backoff.

## 7. Tauri IPC Layer

Located in `storekeeper-app-tauri/src/commands.rs`. Exposes Rust functions to the frontend:

| Command | Purpose |
|---------|---------|
| `get_all_resources` | Return cached resources |
| `refresh_resources` | Trigger manual refresh, return results |
| `get_config` | Load current config from file |
| `save_config` | Write config to file |
| `reload_config` | Re-read config and recreate registries |

Events flow backend → frontend via `app_handle.emit()`:

| Event | Payload | Purpose |
|-------|---------|---------|
| `resources-updated` | `AllResources` | Full resource update after polling |
| `game-resource-updated` | `{ gameId, data }` | Incremental per-game update |
| `refresh-started` | `()` | Manual refresh initiated |
| `daily-reward-claimed` | Claim result | Daily reward claimed |

## 8. Frontend State Management

### Jotai Atom Organization

Located in `frontend/src/modules/`. Uses class-based containers for namespace grouping:

```typescript
export class CoreAtoms {
    readonly resourcesQuery = atomWithQuery(() => resourcesQueryOptions());
    readonly tick = atom((get) => { /* ... */ });
    // ...
}

// Singleton instance
export const atoms = new AtomsContainer();
// Usage: atoms.core.resourcesQuery, atoms.settings.saveConfig
```

### Key Patterns

- **Tick system**: `atomEffect` that runs `setInterval(60s)` to update countdown timers
- **Event listeners**: `atomEffect` that calls `listen()` from Tauri API, updates Query cache on events
- **Query integration**: `atomWithQuery()` bridges Jotai atoms and TanStack Query for Tauri IPC
- **Derived atoms**: Per-game atoms select specific resources from the shared query cache

### Data Update Flow

```
Backend event → atomEffect → setQueryData() → atom re-evaluates → component re-renders
```

See [04-data-flow.md](04-data-flow.md) for complete flow diagrams.

## Design Patterns Summary

| Pattern | Where | Purpose |
|---------|-------|---------|
| Trait Objects | `DynGameClient`, `DynDailyRewardClient` | Type erasure for heterogeneous collections |
| Registry | `GameClientRegistry`, `DailyRewardRegistry` | Dynamic client management |
| Builder | `HttpClientBuilder` | Fluent HTTP client configuration |
| Strategy | Per-game `GameClient` implementations | Pluggable game-specific logic |
| Observer | Tauri event system | Backend → frontend real-time updates |
| Repository | `AppState` with `Arc<RwLock<_>>` | Thread-safe state management |
