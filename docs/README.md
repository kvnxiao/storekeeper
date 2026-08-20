# Documentation

> For prerequisites, commands, and configuration paths, see [DEVELOPMENT.md](../DEVELOPMENT.md). This directory covers architecture and onboarding at a high level; the code is the reference for specifics.

## Architecture

- **[01-overview.md](architecture/01-overview.md)** for the layered design, principles, and the reasoning behind the main technology choices
- **[02-directory-structure.md](architecture/02-directory-structure.md)** for crate layout, the dependency graph, and frontend module conventions
- **[03-core-components.md](architecture/03-core-components.md)** for the concepts whose behaviour is not obvious from one file: registries, state, background tasks, notifications, i18n
- **[04-data-flow.md](architecture/04-data-flow.md)** for how data moves from the game APIs to the UI, and the conventions at each boundary

## Standards

Repository coding rules are stored as skills under `.agents/skills/` and `.claude/skills/`, with reference docs under the `references/` directory for each skill:

- **[SolidJS rules](../.claude/skills/solidjs-rules/)** for reactivity, components, state architecture, data fetching, forms, i18n, testing
- **[Rust rules](../.claude/skills/rust-rules/)** for API design, error handling, testing, lints, workspace conventions
- **[GitHub Actions rules](../.claude/skills/github-actions-rules/)** for workflow versioning and update policy

## Onboarding

- **[01-setup.md](onboarding/01-setup.md)** for development environment setup
- **[02-first-contribution.md](onboarding/02-first-contribution.md)** for adding a game or resource, and the pre-submit checklist
