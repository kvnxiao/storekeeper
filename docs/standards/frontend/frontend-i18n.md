# Frontend Internationalization (i18n)

## Overview

All user-facing UI text must use the i18n system. Hardcoded strings in components are not allowed.

## Framework

**Paraglide JS** (inlang) — compile-time i18n with tree-shakeable message functions.

- Source messages: `frontend/messages/{locale}.json`
- Project config: `frontend/project.inlang/settings.json`
- Compiled output: `frontend/src/paraglide/` (auto-generated, do not edit)
- Base locale: `en`

## Detected Patterns

### Message Import

**Pattern**: Namespace import as `m` from `@/paraglide/messages`.

```typescript
import * as m from "@/paraglide/messages";
```

### Simple Messages

**Pattern**: Call message function directly. Returns a string.

```tsx
<h3>{m.settings_notifications_title()}</h3>
<p>{m.dashboard_no_games()}</p>
```

### Parameterized Messages

**Pattern**: Pass an object with named parameters.

```tsx
// Message: "Failed to load settings: {error}"
<p>{m.settings_failed_to_load({ error: errorMessage })}</p>

// Message: "{name}: {current} of {max}"
<span>{m.stamina_progress_label({ name, current: String(current), max: String(max) })}</span>
```

### Key Naming

**Pattern**: `snake_case` with module prefix.

```json
{
  "settings_notifications_title": "Notifications",
  "settings_notification_minutes_before_full": "Minutes before full",
  "dashboard_refresh_resources": "Refresh resources",
  "resource_resin": "Original Resin",
  "game_genshin_impact": "Genshin Impact"
}
```

| Prefix | Used For | Example |
|--------|----------|---------|
| `settings_` | Settings page UI | `settings_general_title` |
| `dashboard_` | Dashboard page UI | `dashboard_no_games` |
| `game_` | Game display names | `game_honkai_star_rail` |
| `resource_` | Resource display names | `resource_trailblaze_power` |
| `stamina_` | Stamina card UI | `stamina_full_in` |
| `cooldown_` | Cooldown card UI | `cooldown_ready` |
| `textfield_` | Text field UI | `textfield_show_password` |

## Recommended Practices

### Always Use Message Functions

**Status**: Already followed

**Do**:
```tsx
<Button>{m.settings_save()}</Button>
```

**Don't**:
```tsx
<Button>Save Changes</Button>
```

**Why**: Hardcoded strings break localization and make text changes harder to track.

### Prefix Keys by Module

**Status**: Already followed

**Do**:
```json
{
  "settings_general_poll_interval": "Poll Interval (seconds)",
  "settings_general_log_level": "Log Level"
}
```

**Don't**:
```json
{
  "poll_interval": "Poll Interval (seconds)",
  "log_level": "Log Level"
}
```

**Why**: Prefixes prevent key collisions and make it clear where each string is used.

### Use Parameters, Not Concatenation

**Status**: Already followed

**Do**:
```json
{ "settings_failed_to_load": "Failed to load settings: {error}" }
```
```tsx
m.settings_failed_to_load({ error: err.message })
```

**Don't**:
```tsx
`Failed to load settings: ${err.message}`
```

**Why**: Translators need to see the full sentence to translate correctly. Word order varies by language.

### Keep Messages Flat

**Status**: Already followed

**Do**:
```json
{
  "settings_notifications_title": "Notifications",
  "settings_notifications_description": "Configure desktop notifications."
}
```

**Don't**:
```json
{
  "settings": {
    "notifications": {
      "title": "Notifications"
    }
  }
}
```

**Why**: Paraglide JS requires flat key→value pairs. Nested objects are not supported by the inlang message format plugin.

## Adding a New Locale

1. Add the locale code to `frontend/project.inlang/settings.json` → `locales` array
2. Create `frontend/messages/{locale}.json` with all keys translated
3. Add the locale code to `SUPPORTED_LOCALES` in `storekeeper-app-tauri/src/i18n.rs`
4. Create `locales/{locale}.json` with backend-specific translations
5. Register the new locale file in the `load_messages()` match arm in `i18n.rs`

## Backend vs Frontend Messages

The backend and frontend have **separate message catalogs** with different key naming conventions:

| System | File | Key Style | Used For |
|--------|------|-----------|----------|
| Frontend | `frontend/messages/en.json` | `snake_case` flat keys | UI labels, form text |
| Backend | `locales/en.json` | `dot.separated` hierarchical keys | OS notifications, tray menu |

Do not duplicate messages between catalogs. Each system owns its own strings.
