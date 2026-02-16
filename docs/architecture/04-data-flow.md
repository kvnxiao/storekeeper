# Data Flow

Complete data flow through the system, from external game APIs to UI rendering.

## Overview

Primary data paths:

1. **Background Polling** — Periodic resource updates
2. **Manual Refresh** — User-triggered updates
3. **Initial Load** — App startup data fetch
4. **Config Updates** — Settings changes and reloading
5. **Daily Reward Claiming** — Automated and manual claiming
6. **Notification Checking** — Background resource alert evaluation
7. **Locale Switching** — Language change propagation

## 1. Background Polling Flow

```mermaid
sequenceDiagram
    participant PL as Polling Loop
    participant ST as AppState
    participant REG as GameClientRegistry
    participant GC as GameClient
    participant API as External API
    participant EV as Event System
    participant FE as Frontend

    PL->>ST: is_refreshing()?
    ST-->>PL: false
    PL->>ST: set_refreshing(true)
    PL->>ST: fetch_all_resources()
    ST->>REG: fetch_all()

    par HoYoLab (sequential within)
        REG->>GC: fetch_resources_json() [Genshin]
        GC->>API: HTTP GET with auth
        API-->>GC: JSON response
        GC-->>REG: resources
        REG->>EV: emit("game-resource-updated")
        EV->>FE: incremental update
        REG->>GC: fetch_resources_json() [HSR]
        GC->>API: HTTP GET with auth
        API-->>GC: JSON response
        GC-->>REG: resources
        REG->>EV: emit("game-resource-updated")
        EV->>FE: incremental update
    and Kuro (parallel with HoYoLab)
        REG->>GC: fetch_resources_json() [Wuwa]
        GC->>API: HTTP GET with auth
        API-->>GC: JSON response
        GC-->>REG: resources
        REG->>EV: emit("game-resource-updated")
        EV->>FE: incremental update
    end

    REG-->>ST: HashMap<GameId, Value>
    ST->>ST: set_resources()
    ST->>ST: set_refreshing(false)
    ST->>EV: emit("resources-updated")
    EV->>FE: full update
    FE->>FE: Update Query Cache + Re-render
```

### Key Steps

1. **Polling loop** wakes after `poll_interval_secs` (configurable, default 300s)
2. **Guard check**: Skip if already refreshing or no clients configured
3. **Registry groups** games by `ApiProvider`, fetches providers in parallel
4. **Within each provider**, games are fetched sequentially (rate limit safety)
5. **Per-game event** emitted immediately after each game completes
6. **Full update event** emitted after all games finish
7. **Frontend** receives events via Jotai effect atoms, updates TanStack Query cache

## 2. Manual Refresh Flow

```mermaid
sequenceDiagram
    participant UI as Frontend UI
    participant Q as TanStack Query
    participant IPC as Tauri IPC
    participant CMD as Tauri Command
    participant POLL as Polling Module
    participant ST as AppState

    UI->>Q: mutate()
    Q->>IPC: invoke("refresh_resources")
    IPC->>CMD: refresh_resources()
    CMD->>POLL: refresh_now()
    POLL->>ST: emit("refresh-started")
    POLL->>ST: fetch_all_resources()
    Note over ST: Same as background polling
    ST-->>POLL: AllResources
    POLL-->>IPC: AllResources
    IPC-->>Q: AllResources
    Q-->>UI: Updated data
```

Differences from background polling:
- **Returns data** synchronously to the caller
- **Emits `refresh-started`** so UI can show loading state immediately
- **Rejects** if already refreshing (returns error string)

## 3. Initial Load Flow

```mermaid
sequenceDiagram
    participant APP as Tauri App
    participant CFG as Config Files
    participant ST as AppState
    participant REG as Registry
    participant I18N as i18n Module
    participant NOTIF as Notification Checker
    participant POLL as Polling

    APP->>CFG: Load config.toml + secrets.toml
    CFG-->>APP: AppConfig, SecretsConfig
    APP->>REG: create_registry(config, secrets)
    REG-->>APP: GameClientRegistry
    APP->>ST: Initialize AppState (with NotificationTracker)
    APP->>I18N: init(config.general.language)
    APP->>POLL: start_polling(cancel_token)
    APP->>NOTIF: start_notification_checker(cancel_token)
    POLL->>POLL: Sleep 2s
    POLL->>ST: poll_resources()
    Note over ST: First fetch populates state
```

Timeline:
- **T+0ms**: Tauri app starts, config loaded, state initialized, i18n initialized
- **T+2000ms**: First resource fetch (background)
- **T+~3000ms**: Frontend receives first `resources-updated` event
- **T+60000ms**: First notification check runs (reads cached resources)

## 4. Config Update Flow

```mermaid
sequenceDiagram
    participant UI as Settings UI
    participant IPC as Tauri IPC
    participant FS as File System
    participant ST as AppState
    participant I18N as i18n Module
    participant TRAY as System Tray

    UI->>IPC: invoke("save_config", config)
    IPC->>FS: Write config.toml
    FS-->>IPC: Ok

    UI->>IPC: invoke("reload_config")
    IPC->>FS: Read config.toml + secrets.toml
    FS-->>IPC: AppConfig, SecretsConfig
    IPC->>ST: Recreate registries
    IPC->>ST: Clear notification tracker
    IPC->>I18N: set_locale(config.language)
    IPC->>TRAY: Rebuild tray menu (localized labels)
    IPC->>ST: refresh_now()
    Note over ST: Immediate fetch with new config
```

`reload_config()` recreates both `GameClientRegistry` and `DailyRewardRegistry` from the new config, clears notification cooldowns, updates the backend locale, rebuilds the tray menu, then triggers an immediate refresh.

## 5. Daily Reward Claiming Flow

```mermaid
sequenceDiagram
    participant SCH as Scheduled Task
    participant DREG as DailyRewardRegistry
    participant DC as DailyRewardClient
    participant API as HoYoLab API
    participant NOT as Notification

    SCH->>DC: get_reward_info()
    DC->>API: GET /info
    API-->>DC: { is_sign: false }

    SCH->>DC: claim_daily_reward()
    DC->>API: POST /sign
    API-->>DC: { retcode: 0 }

    DC->>DC: get_reward_status()
    DC->>API: GET /home
    API-->>DC: { awards: [...] }

    DC-->>SCH: ClaimResult::success(reward, info)
    SCH->>NOT: Desktop notification
```

Two phases:
1. **Startup claims**: Run once on app start, claim any unclaimed rewards
2. **Scheduled loop**: Calculate next claim time, sleep until then, claim with retry

Retry on transient errors with exponential backoff (3 retries, 500ms base, 30s max).

## 6. Notification Checking Flow

```mermaid
sequenceDiagram
    participant NC as Notification Checker (60s timer)
    participant ST as AppState
    participant TK as NotificationTracker
    participant I18N as i18n Module
    participant NP as tauri-plugin-notification
    participant OS as OS Toast

    NC->>ST: get_resources() (read lock)
    ST-->>NC: AllResources (cached)
    NC->>ST: get_game_notification_config(game_id)
    ST-->>NC: HashMap<resource_type, NotificationConfig>

    loop For each (game, resource) with enabled config
        NC->>NC: extract_resource_info(data)
        Note over NC: Detect: StaminaResource / CooldownResource / ExpeditionResource
        NC->>TK: should_notify(game, resource, config, info, now)
        alt In window + cooldown expired
            TK-->>NC: true
            NC->>I18N: t_args("notification.title", ...)
            NC->>I18N: build_notification_body(...)
            NC->>NP: notification().builder().title().body().show()
            NP->>OS: Display toast notification
            NC->>TK: record(game, resource, now)
        else Not in window OR within cooldown
            TK-->>NC: false
        end
    end
```

### Key Design Choices

- **No API calls**: Reads only cached state. Notification accuracy depends on polling freshness.
- **60-second check interval**: Balances responsiveness with CPU usage. Resource timers are minute-granularity anyway.
- **Separate from polling**: The notification checker runs on its own timer, independent of the polling loop. This means notifications keep checking even if a poll cycle takes longer than expected.
- **Write lock scope**: A single write lock is acquired for the entire check cycle to update the tracker. Read locks are released before the write lock is acquired.

## 7. Locale Switching Flow

```mermaid
sequenceDiagram
    participant UI as Settings UI
    participant IPC as Tauri IPC
    participant CFG as Config (TOML)
    participant I18N as Backend i18n
    participant TRAY as System Tray
    participant NC as Notification Checker

    UI->>IPC: save_config({ general: { language: "en" } })
    IPC->>CFG: Write config.toml
    UI->>IPC: reload_config()
    IPC->>I18N: set_locale("en")
    Note over I18N: Replace global Messages store
    IPC->>TRAY: build_tray_menu()
    Note over TRAY: Labels now use new locale
    Note over NC: Next check cycle uses new locale strings
```

**Frontend locale switching**: Paraglide JS handles frontend locale independently. The frontend reads `config.general.language` and sets the Paraglide runtime locale. Message functions automatically return strings in the active locale.

**What gets re-localized on language change**:
- System tray menu labels (rebuilt immediately)
- Future OS notification text (next check cycle)
- Frontend UI text (React re-render with new message functions)

## Data Transformations

### API Response → Frontend UI

```
1. External API JSON response
   { "current_resin": 150, "resin_recovery_time": "1708020000" }
       │
       ▼
2. Rust deserialization (game-specific response struct)
   DailyNoteResponse { current_resin: 150, resin_recovery_time: ... }
       │
       ▼
3. Transform to core resource type
   StaminaResource { current: 150, max: 160, full_at: DateTime, regen_rate_seconds: 480 }
       │
       ▼
4. Wrap in game-specific enum
   GenshinResource::Resin(StaminaResource { ... })
       │
       ▼
5. Serialize to JSON (type erasure at DynGameClient boundary)
   { "type": "resin", "data": { "current": 150, "max": 160, "fullAt": "...", "regenRateSeconds": 480 } }
       │
       ▼
6. Store in AllResources
   { "games": { "GENSHIN_IMPACT": [...], "HONKAI_STAR_RAIL": [...] }, "lastUpdated": "..." }
       │
       ├──▶ Tauri event → Frontend
       │
       └──▶ Notification checker reads cached data (no transformation)
            └── extract_resource_info() → ResourceInfo { completion_at, is_complete, current, max }
```

### Naming Convention at Boundaries

| Layer | Convention | Example |
|-------|-----------|---------|
| Rust structs | snake_case | `current_resin`, `full_at` |
| JSON serialization | camelCase | `fullAt`, `regenRateSeconds` |
| GameId enum | SCREAMING_SNAKE_CASE | `GENSHIN_IMPACT` |
| TypeScript interfaces | camelCase | `fullAt`, `regenRateSeconds` |
| Config/Secrets TOML | snake_case | `poll_interval_secs` |
| Backend i18n keys | dot.separated | `notification.resource_full` |
| Frontend i18n keys | snake_case | `settings_notifications_title` |

Serde's `#[serde(rename_all = "camelCase")]` handles the Rust↔JSON conversion automatically. See [frontend-tauri-bridge.md](../standards/frontend/frontend-tauri-bridge.md) for full details.

## Rate Limiting Strategy

```
┌─────────────────────────────────────────┐
│ HoYoLab provider (~1 req/sec limit)    │
│   Genshin ──→ HSR ──→ ZZZ             │
│         (sequential, no overlap)        │
└─────────────────────────────────────────┘
         ║ parallel (independent limits)
┌─────────────────────────────────────────┐
│ Kuro provider                           │
│   Wuwa ────────────────────────────────│
└─────────────────────────────────────────┘
```

Implementation: `join_all()` for parallel providers, sequential `for` loop within each provider. Daily reward claims add a 500ms delay between games.

## Error Handling

```
API Error
    ├── HTTP Error (network, timeout)
    │    └── reqwest-retry middleware retries with exponential backoff
    │
    ├── API Response Error (retcode != 0)
    │    └── ClientError::ApiError { code, message }
    │
    └── Game Client Error
         └── Type-erased Box<dyn Error>
              └── Logged via tracing::warn!, game skipped in results

Notification Error
    └── Failed to send OS notification
         └── Logged via tracing::warn!, cooldown NOT recorded (retries next cycle)
```

Failed game fetches don't crash the app or block other games. The UI shows stale data for the failed game until the next successful fetch.

## Timing Characteristics

| Event | Typical Latency |
|-------|----------------|
| Background poll | ~1-3s (depends on enabled games) |
| Manual refresh | ~1-3s (same, but blocks UI with loading) |
| Config reload | ~50-100ms (file I/O + registry recreation) |
| Daily reward claim | ~500ms-1s (single API call) |
| Notification check | <10ms (reads cached state only) |
| Event propagation (backend → frontend) | <10ms (in-process IPC) |
| UI update | <16ms (single React render frame) |
