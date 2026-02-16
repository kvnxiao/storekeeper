# First Contribution

This guide walks through common contribution patterns. See [DEVELOPMENT.md](../../DEVELOPMENT.md) for architecture and conventions reference.

## Adding a New Game

The most common contribution is adding support for a new game. The architecture is designed to make this straightforward.

### 1. Create the Game Crate

```bash
cargo init storekeeper-game-{name} --lib
```

Follow the standard structure:

```
storekeeper-game-{name}/src/
├── lib.rs          # Public exports
├── client.rs       # GameClient implementation
├── resource.rs     # Game-specific resource enum
└── error.rs        # Error types using thiserror
```

### 2. Define Resources

Create a tagged enum wrapping the core resource types:

```rust
use serde::{Deserialize, Serialize};
use storekeeper_core::resource::StaminaResource;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum NewGameResource {
    Stamina(StaminaResource),
    // Add variants for each tracked resource
}
```

### 3. Implement GameClient

Implement the `GameClient` trait from `storekeeper-core`:

```rust
use async_trait::async_trait;
use storekeeper_core::game::{GameClient, GameId};

#[async_trait]
impl GameClient for NewGameClient {
    type Resource = NewGameResource;
    type Error = Error;

    fn game_id(&self) -> GameId { GameId::NewGame }
    fn game_name(&self) -> &'static str { "New Game" }

    async fn fetch_resources(&self) -> Result<Vec<Self::Resource>, Self::Error> {
        // Call the game's API and transform to resource types
    }

    async fn is_authenticated(&self) -> Result<bool, Self::Error> {
        // Check if credentials are valid
    }
}
```

If the game uses a new API provider, create a `storekeeper-client-{provider}` crate first. Otherwise, use an existing client crate.

### 4. Register in the App

Update `storekeeper-app-tauri/src/clients.rs` to create the client from config:

```rust
if let Some(ref cfg) = config.games.new_game {
    if cfg.enabled {
        if let Ok(client) = NewGameClient::new(/* credentials */) {
            registry.register(Box::new(client));
        }
    }
}
```

Add the `GameId` variant in `storekeeper-core/src/game_id.rs` and wire up config fields.

### 5. Add Frontend Components

Create `frontend/src/modules/games/{name}/`:

```
{name}/
├── components/
│   └── {Name}Section.tsx    # Main section component
└── {name}.atoms.ts          # Atoms to select resources (if needed)
```

Add the section to the dashboard in `frontend/src/routes/index.tsx`.

### 6. Update Configuration

Update `default_config_content()` in `storekeeper-core/src/config.rs` and the config types.

## Modifying an Existing Game

To add a new tracked resource to an existing game:

1. Add the API response field to the game's response struct
2. Add a variant to the game's resource enum
3. Map the API response to the new resource in `fetch_resources()`
4. Add a frontend component to display it

## Working with the Frontend

Key patterns to follow:
- **State**: Use Jotai atoms. See [frontend-atoms.md](../standards/frontend/frontend-atoms.md)
- **Queries**: Use TanStack Query options. See [frontend-queries.md](../standards/frontend/frontend-queries.md)
- **Components**: Use React Aria Components. See [frontend-react-aria.md](../standards/frontend/frontend-react-aria.md)
- **Styling**: Use Tailwind CSS with `tv()`. See [frontend-styling.md](../standards/frontend/frontend-styling.md)

## Checklist Before Submitting

- [ ] `just fix` passes (Rust linting + formatting)
- [ ] `just fix-web` passes (frontend linting + formatting)
- [ ] `just test` passes (Rust tests)
- [ ] No `unwrap()` or `expect()` in non-test code
- [ ] Error types use `thiserror` with descriptive messages
- [ ] New public APIs have documentation comments
