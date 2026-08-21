# First Contribution

Where to make changes for the common tasks. Follow the shape of the existing code in each place rather than the outline here; the closest existing game or component is the best template.

## Adding a New Game

Adding a game touches both sides but changes no shared logic. Registries, polling, notifications, and state work with any game that implements the traits.

**Backend**

1. **New crate**: `storekeeper-game-{name}`, added to the workspace members. Copy the structure of an existing game crate: client, resource enum, errors.
2. **API client**: if the provider is new, add a `storekeeper-client-{provider}` crate for authentication and requests. Otherwise reuse the existing provider client.
3. **Resources**: map the API response into the shared resource shapes, wrapped in the tagged resource enum for the game. Add the matching resource identifiers used as config keys.
4. **Identity**: add the game to the game identifier enum and to its provider grouping. The grouping is what gives the game correct rate limit behaviour.
5. **Config**: add the game config section and its defaults, including whether it supports daily rewards.
6. **Registration**: register the client from config in the Tauri client factory.

**Frontend**

7. **Module**: add `frontend/src/modules/games/{name}/` with a section component and its resource selectors.
8. **Wiring**: render the section on the dashboard and add a settings section for it. Reuse the shared game settings section if the game config matches an existing provider shape.
9. **Config mirror**: mirror the new config section in the settings types, snake_case, matching the Rust names exactly.

**Shared**

10. **Locale strings**: add the game name and one entry per resource to every file in `locales/`. Both the backend and the frontend read them.
11. **Icons**: add resource icons under `frontend/public/` and register their paths with the other per-resource metadata.

## Adding a Resource to an Existing Game

1. Extend the API response and map the new field into a resource variant.
2. Add the matching resource identifier so it can be tracked and configured.
3. Add its locale entry and icon.
4. Render it from the game section component using the shared resource cards.

## Working with the Frontend

Read the [`solidjs-rules`](../../.claude/skills/solidjs-rules/) skill from `.agents/skills/` or `.claude/skills/` before touching frontend code; it covers reactivity, component conventions, state architecture, data fetching, forms, i18n, and testing. The short version:

- Business state and logic go in state modules, not components. Components read accessors and render.
- Server state stays in the query cache. Modules export query and mutation options; components consume them.
- Values derived from elapsed time read the app-wide tick instead of starting their own timer.
- Use the shared UI kit and Kobalte primitives before writing new markup, and `tv()` variants for styling.
- File names follow the `<module>.<type>.ts` convention in [02-directory-structure.md](../architecture/02-directory-structure.md).

For Rust, read the [`rust-rules`](../../.claude/skills/rust-rules/) skill from `.agents/skills/` or `.claude/skills/`.

## Checklist Before Submitting

- [ ] `just fix` passes (Rust linting and formatting)
- [ ] `just fix-web` passes (frontend linting, formatting, type checking)
- [ ] `just test` passes
- [ ] `just test-web` passes
- [ ] No `unwrap()` or `expect()` outside tests
- [ ] Error types use `thiserror` with descriptive messages
- [ ] New public APIs have documentation comments
