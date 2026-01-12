# Tauri-Frontend Bridge Conventions

This document describes the conventions for data exchange between the Rust backend and TypeScript frontend via Tauri commands.

## Naming Conventions

| Context | Convention | Example |
|---------|------------|---------|
| Config/Secrets (Tauri ↔ Frontend) | snake_case | `poll_interval_secs`, `auto_claim_daily_rewards` |
| Resource types | camelCase | `fullAt`, `regenRateSeconds`, `isReady` |
| GameId enum | SCREAMING_SNAKE_CASE | `GENSHIN_IMPACT`, `HONKAI_STAR_RAIL` |
| TOML config files | snake_case | `poll_interval_secs`, `auto_claim_daily_rewards` |

## Rationale

The frontend generally uses camelCase for JavaScript convention. However, **config and secrets types use snake_case** as an exception for two reasons:

1. **No DTO duplication** - Rust types serialize directly without intermediate DTOs
2. **Consistency with TOML** - Config files use snake_case, and the same data flows through Tauri unchanged

Resource types (`StaminaResource`, `CooldownResource`, etc.) use camelCase because they have `#[serde(rename_all = "camelCase")]` in Rust.

## How to Identify snake_case Types

In frontend code, types from Rust config fall into two categories:

**snake_case (config/secrets):**
- `AppConfig`, `GeneralConfig`, `NotificationConfig`, `ThresholdConfig`
- `GamesConfig`, `GenshinConfig`, `HsrConfig`, `ZzzConfig`, `WuwaConfig`
- `SecretsConfig`, `HoyolabSecrets`, `KuroSecrets`

**camelCase (resources):**
- `StaminaResource`, `CooldownResource`, `ExpeditionResource`
- `AllResources`, `AllDailyRewardStatus`

## Example

```typescript
// Config types use snake_case
const config: AppConfig = {
  general: {
    poll_interval_secs: 300,
    start_minimized: true,
    log_level: "info",
  },
  // ...
};

// Resource types use camelCase
const resource: StaminaResource = {
  current: 120,
  max: 200,
  fullAt: "2024-01-15T10:30:00Z",
  regenRateSeconds: 480,
};
```

## Adding New Config Fields

1. Add field to Rust type in `storekeeper-core/src/config.rs` (snake_case)
2. Add field to TypeScript interface in `frontend/src/modules/settings/settings.types.ts` (snake_case)
3. Update UI components to use the new field (snake_case)

No DTO conversion needed - the data flows directly.

## Adding New Resource Fields

1. Add field to Rust type in the appropriate game crate (snake_case)
2. Ensure `#[serde(rename_all = "camelCase")]` is on the struct for frontend compatibility
3. Add field to TypeScript interface in `frontend/src/modules/resources/resources.types.ts` (camelCase)
4. Update UI components in `frontend/src/modules/resources/components/` to use the new field
