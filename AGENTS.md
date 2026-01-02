# Critical Instructions

**IMPORTANT**: Before starting any work, run `beans prime` and follow its instructions.

**IMPORTANT**: before completing any task, you MUST run the following commands and address any issues:

```bash
cargo clippy --workspace --all-targets --all-features
cargo fmt --check
```

- Fix all clippy warnings and errors before marking work as done
- Run `cargo fmt` to format code if `cargo fmt --check` reports formatting issues
- These checks are mandatory for every code change, no exceptions

## Storekeeper

Storekeeper is a desktop application that provides real-time stamina resource tracking for various gacha games such as Genshin Impact, Honkai Star Rail, Zenless Zone Zero, Wuthering Waves, etc.

See [@DEVELOPMENT.md](DEVELOPMENT.md) for architecture, conventions, and development setup. Also refer to the [@justfile](justfile) for a list of available project commands, and the workspace [@Cargo.toml](Cargo.toml) to see how the project is configured and what dependencies are being used.

### Rust Guidelines

General Rust development guidelines are in `~/.claude/rules/`:

- `rust-linting.md` - Clippy configuration
- `rust-error-handling.md` - Error types with thiserror/anyhow
- `rust-defensive-programming.md` - Input validation, builders, newtypes
- `rust-code-quality.md` - Code organization, enums, parameter types
- `rust-unsafe.md` - Avoiding unsafe code
- `rust-testing.md` - Test organization
- `rust-documentation.md` - Doc requirements
- `rust-performance.md` - Performance best practices
- `rust-dependencies.md` - Dependency management
- `rust-workspaces.md` - Multi-crate workspace patterns
