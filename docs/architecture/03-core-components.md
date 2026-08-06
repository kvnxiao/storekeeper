# Core Concepts

The parts of the system whose behaviour is not obvious from reading a single file. Everything here is about intent and invariants; the code is the reference for signatures and fields.

## Game Abstraction

Two traits define what a game can do: fetching resources, and claiming daily rewards. They are separate because not every game supports daily rewards and because the two have different lifecycles, one polled on an interval and one claimed once a day.

Implementations carry their own resource and error types, so they cannot be stored together. The application layer therefore works with type-erased versions that hand back JSON, which is also the form it forwards to the frontend. The consequence worth remembering: once past the game crate boundary, the backend treats resources as opaque data and never re-parses them into typed values.

## Resource Model

Three shared resource shapes cover every game: regenerating resources, one-time cooldowns, and timed dispatches. Each game wraps them in a tagged enum, which serializes as a discriminated union so the frontend can match on the resource kind.

Resource kinds appear twice on purpose: as data (what was fetched) and as identifiers (config keys for tracked resources and notification settings). Both serialize to the same strings, so a config key always names a real resource.

## HTTP and Authentication

The infrastructure crate owns client construction and the retry policy so no game or provider crate reimplements backoff. Retries apply only to transient failures; an API-level rejection is a real error and surfaces immediately.

The two providers authenticate differently. HoYoLab needs user-supplied cookie credentials plus a per-request signature. Kuro reads credentials from the game launcher's local cache, so Wuthering Waves needs no manual credential entry as long as the user has signed into the launcher.

## Registries and Fetch Strategy

Registries hold the enabled game clients, rebuilt from config whenever the settings that determine client setup change. Fetching groups clients by API provider: sequential within a provider, parallel across providers. HoYoLab games share a rate limit, so overlapping their requests causes throttling; Kuro is independent, so it runs alongside.

Each game emits its own update event as soon as it finishes, before the batch completes, so the dashboard fills in progressively instead of waiting for the slowest game.

## Application State

One lock guards the shared state: cached resources, registries, daily reward status, config and secrets, and notification cooldowns. Multiple readers or one writer, all async.

Two things deliberately sit outside that lock:

- The refresh flag is atomic, so a caller can claim the right to refresh without waiting on the write lock. This is what makes overlapping fetches impossible.
- The auto-claim scheduler waits on a notification handle, so a settings change can wake it immediately rather than after its current sleep.

Secrets are kept in memory alongside config only so a save can be diffed against the previous state without re-reading files.

## Background Tasks

Three independent tokio tasks, each cancellable at shutdown:

**Polling** refreshes resources on the configured interval, after a short startup delay, skipping the cycle if a refresh is already running or no games are configured.

**Scheduled claims** claims daily rewards at each game's configured time. Sleeps are chunked so a claim time is not missed when the OS suspends, and the task also claims anything outstanding at startup. Failures retry with backoff.

**Notification checking** runs on its own minute timer and reads cached state only. Notification accuracy therefore follows polling freshness, which is the intended trade: alerting never makes network calls.

Keeping these independent means a slow poll cycle cannot delay a claim or an alert.

## Notifications

Per resource, a user can notify on one of two mutually exclusive thresholds: a lead time before the resource completes, or a value the resource reaches (regenerating resources only). With neither set, the notification fires on completion.

Cooldown semantics are the subtle part:

- A positive cooldown re-notifies on that interval while the resource stays inside the notification window.
- A zero cooldown notifies once per entry into the window.
- Leaving the window clears the cooldown, so re-entering notifies again. Spending stamina and regenerating past the threshold is a fresh alert, not a suppressed one.
- Saving settings clears cooldowns only for games whose notification config changed.

The checker identifies a resource's kind from the shape of its JSON rather than carrying type information through the erasure boundary. Notification text comes from the shared locale catalog, with durations and clock times formatted for the active locale rather than assembled by hand.

Users can send a preview notification from settings; it uses cached data when available so the preview matches what a real alert would look like.

## IPC Surface

Commands cover reading resources, reading and saving settings, daily reward status and claiming, and locale queries. Events cover refresh started, a single game updated, all resources updated, and a daily reward claimed. Commands return typed error codes rather than opaque strings, so the frontend can branch on the failure.

Event names are the one contract that both sides hardcode, so each side declares them once and a Rust test asserts the frontend's declarations still match the backend's.

### Saving Settings Applies the Minimum

Saving is a single command that writes both config files and then applies only what changed, diffed in memory against the state snapshot. What changed determines what happens: a locale change switches the backend locale and rebuilds the tray, an autostart change syncs the OS integration, and only games whose client-relevant settings changed get new clients and a refetch.

This matters because the alternative (rebuild everything and refetch) turns every settings save into API calls, which the providers rate limit. Editing a notification threshold costs nothing on the network.

## Internationalization

One catalog per locale, shared by both runtimes.

The backend embeds catalogs at compile time and holds the active locale in a global that can be switched at runtime, which is what lets a language change take effect without a restart. It parses ICU MessageFormat and uses ICU4X for plurals, durations, and times.

The frontend compiles the same catalog into tree-shakeable message functions at build time. Paraglide restores the persisted locale at startup, then the app asks the backend which locale is actually in effect and follows it, since a config value of "no language set" resolves against the system locale on the Rust side.

## Frontend State Ownership

Two homes, split by what owns the truth:

- **Server state** lives in the query cache. Query and mutation options are defined per module; components consume them. Backend events write into the cache directly, so pushed updates and fetched updates land in the same place.
- **Client state** lives in state modules: module-scope Solid singletons that export accessors and named actions and never their setters. This keeps business logic testable without rendering and gives every write a name worth grepping for.

Components hold only view-local state. Values that derive from the passage of time read the app-wide tick rather than starting their own timers.

## Patterns in Use

| Pattern | Where | Purpose |
|---|---|---|
| Trait objects | Game and daily reward clients | Heterogeneous collections behind one interface |
| Registry | Game and daily reward registries | Config-driven client lifecycle |
| Builder | HTTP client construction | Shared transport configuration |
| Strategy | Per-game client implementations | Pluggable game-specific logic |
| Observer | Tauri events | Backend to frontend updates without polling |
| Cooldown tracker | Notification state | Deduplicating OS notifications |
| Runtime-switchable singleton | Backend locale store | Language changes without restart |
