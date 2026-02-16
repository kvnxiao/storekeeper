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
    pub notification_tracker: NotificationTracker,
}
```

**Concurrency model**: `Arc<RwLock<_>>` allows multiple concurrent readers (Tauri commands) or a single writer (polling updates). All access is async (`.read().await`, `.write().await`).

**Access patterns**:
- Tauri commands: `state: State<'_, AppState>`
- Background tasks: `app_handle.state::<AppState>()`

**Config reload**: `reload_config()` re-reads TOML files, recreates registries, clears the notification tracker, and updates the i18n locale — all without restarting the app.

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

Retries on transient errors with exponential backoff (3 retries, 500ms base, 30s max).

### Notification Checker

Located in `storekeeper-app-tauri/src/notification.rs`. Runs on a separate 60-second timer. **Does not make API calls** — reads cached resources from state only.

```
Every 60 seconds:
  ├── Read cached resources from AppState (read lock)
  ├── Read per-game notification configs from AppState
  ├── For each (game, resource) pair with notifications enabled:
  │    ├── Extract timing info (fullAt, readyAt, earliestFinishAt)
  │    ├── Check if resource is in notification window
  │    ├── Check cooldown tracker (write lock)
  │    └── Send OS toast notification if conditions met
  └── Record notification timestamp for cooldown tracking
```

See [04-data-flow.md](04-data-flow.md) for the complete notification flow.

## 7. Notification System

### ResourceNotificationConfig

Located in `storekeeper-core/src/config.rs`. Per-resource notification settings stored in each game's config section:

```rust
pub struct ResourceNotificationConfig {
    pub enabled: bool,
    pub notify_minutes_before_full: Option<u32>,  // Minutes-before-full mode
    pub notify_at_value: Option<u64>,              // Value-threshold mode (stamina only)
    pub cooldown_minutes: u32,                     // Minutes between repeated notifications
}
```

**Two threshold modes** (mutually exclusive):
- **Minutes before full**: Fire when time-to-completion drops below N minutes. Works for all resource types.
- **At value**: Fire when resource value reaches N. Converts to time-based comparison using the regen rate. Stamina resources only.

If both are `None`, notifications fire only when the resource is full/ready.

### NotificationTracker

Located in `storekeeper-app-tauri/src/notification.rs`. Tracks cooldown state per `(GameId, resource_type)` pair.

```rust
pub struct NotificationTracker {
    cooldowns: HashMap<(GameId, String), DateTime<Utc>>,
}
```

**Cooldown behavior**:
- `cooldown_minutes > 0`: Re-notify every N minutes while the resource stays in the notification window
- `cooldown_minutes == 0`: Notify once per window entry, no repeats until the resource leaves and re-enters the window
- When a resource leaves the notification window (e.g., stamina consumed), the cooldown is cleared. Re-entering triggers a fresh notification.
- On config reload, all cooldowns are cleared to prevent stale state.

### Resource Info Extraction

The notification checker detects resource kind by JSON field presence (no type information needed):
- Has `fullAt` + `current` + `max` → `StaminaResource`
- Has `readyAt` + `isReady` → `CooldownResource`
- Has `earliestFinishAt` → `ExpeditionResource`

### Notification Messages

Notification title and body are built using the backend i18n system with ICU MessageFormat:

```
Title: "{game_name} — {resource_name}"
Body (before full): "{resource_name} will be full in {minutes, plural, one {# minute} other {# minutes}}"
Body (full): "{resource_name} is full!"
Body (overdue): "{resource_name} has been full for {minutes, plural, one {# minute} other {# minutes}}"
Body (value mode): "{resource_name} has reached {current}/{max}"
```

### Preview Notifications

The `send_preview_notification` Tauri command lets users test notifications from the settings UI. It uses cached resource data to build a realistic notification body, or falls back to a "no data" message if the resource hasn't been fetched yet.

## 8. Tauri IPC Layer

Located in `storekeeper-app-tauri/src/commands.rs`. Exposes Rust functions to the frontend:

| Command | Purpose |
|---------|---------|
| `get_all_resources` | Return cached resources |
| `refresh_resources` | Trigger manual refresh, return results |
| `get_config` | Load current config from file |
| `save_config` | Write config to file |
| `get_secrets` | Load current secrets from file |
| `save_secrets` | Write secrets to file |
| `reload_config` | Re-read config, recreate registries, update locale |
| `open_config_folder` | Open config directory in file manager |
| `send_preview_notification` | Send test OS notification for a resource |
| `get_daily_reward_status` | Return cached daily reward status |
| `refresh_daily_reward_status` | Fetch fresh daily reward status |
| `claim_daily_rewards` | Claim all pending daily rewards |
| `claim_daily_reward_for_game` | Claim daily reward for one game |
| `get_daily_reward_status_for_game` | Get status for one game |
| `get_supported_locales` | Return list of supported locale codes |

Events flow backend → frontend via `app_handle.emit()`:

| Event | Payload | Purpose |
|-------|---------|---------|
| `resources-updated` | `AllResources` | Full resource update after polling |
| `game-resource-updated` | `{ gameId, data }` | Incremental per-game update |
| `refresh-started` | `()` | Manual refresh initiated |
| `daily-reward-claimed` | Claim result | Daily reward claimed |

## 9. Internationalization (i18n)

### Backend i18n Module

Located in `storekeeper-app-tauri/src/i18n.rs`. Provides localized strings for OS notifications and system tray labels.

**Architecture**:
- Locale JSON files are embedded at compile time via `include_str!()` from `locales/*.json`
- Messages stored in a global `OnceLock<RwLock<Messages>>` — initialized once, switchable at runtime
- ICU4X `PluralRules` created on-demand per `t_args` call (not stored, because `PluralRules` is `!Send + !Sync`)

**API**:

```rust
// Simple lookup — returns the key itself if not found
i18n::t("tray.quit") // → "Quit"

// Substitution with plural support
i18n::t_args("notification.resource_full_in", &[
    ("resource_name", Value::from("Original Resin")),
    ("minutes", Value::Number(45)),
]) // → "Original Resin will be full in 45 minutes"
```

**Message format**: ICU MessageFormat syntax with `{name}` for simple substitution and `{name, plural, one {...} other {...}}` for plural dispatch. `#` in plural branches is replaced by the count value.

**Locale switching**: `i18n::set_locale("en")` replaces the message store at runtime. Called during config reload to pick up language changes. Also rebuilds the tray menu with new locale strings.

**Key naming convention** (backend `locales/*.json`):
- `notification.*` — Notification title/body templates
- `tray.*` — System tray menu labels
- `game.{short_id}.name` — Game display names
- `game.{short_id}.resource.{type}` — Resource display names
- `resource.unknown` — Fallback for unknown resources

### Frontend i18n (Paraglide JS)

Located in `frontend/messages/*.json` (source) and `frontend/src/paraglide/` (compiled output).

**Architecture**:
- Source messages in `frontend/messages/en.json` using inlang message format
- `project.inlang/settings.json` configures locales and message path pattern
- Paraglide JS compiles messages at build time into tree-shakeable function exports
- Generated code in `frontend/src/paraglide/` (gitignored, do not edit manually)

**Usage in components**:

```typescript
import * as m from "@/paraglide/messages";

// Simple message
<h3>{m.settings_notifications_title()}</h3>

// Message with parameters
<p>{m.settings_failed_to_load({ error: errorMessage })}</p>
```

**Key naming convention** (frontend `messages/en.json`):
- `snake_case` with module prefix: `settings_notifications_title`, `dashboard_refresh_resources`
- Parameter interpolation: `{error}`, `{name}`, `{current}`, `{max}`
- Game names: `game_genshin_impact`, `game_honkai_star_rail`
- Resource names: `resource_resin`, `resource_trailblaze_power`

See [frontend-i18n.md](../standards/frontend/frontend-i18n.md) for frontend i18n standards.

## 10. Frontend State Management

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
| Cooldown Tracker | `NotificationTracker` | Dedup/rate-limit OS notifications |
| Global Singleton | `i18n::MESSAGES` with `OnceLock<RwLock<_>>` | Runtime-switchable locale store |
