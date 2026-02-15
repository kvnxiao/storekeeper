# Documentation

> For a quick-start reference (prerequisites, commands, configuration), see [DEVELOPMENT.md](../DEVELOPMENT.md). This `docs/` directory is the comprehensive source of truth for architecture, standards, and onboarding.

## Architecture

System design, component relationships, and data flow:

- **[01-overview.md](architecture/01-overview.md)** — Layered architecture, design decisions, technology stack
- **[02-directory-structure.md](architecture/02-directory-structure.md)** — Crate layout, dependency graph, frontend module structure
- **[03-core-components.md](architecture/03-core-components.md)** — Traits, registries, state management, design patterns
- **[04-data-flow.md](architecture/04-data-flow.md)** — API to UI data flow, polling, events, rate limiting

## Standards

Actionable patterns and conventions for both humans and AI agents:

- **[Frontend Standards](standards/frontend/)** — React, TypeScript, Tailwind, React Aria conventions
- **[Rust Standards](standards/rust/)** — Linting, error handling, testing, workspace conventions

## Onboarding

Getting started guides:

- **[01-setup.md](onboarding/01-setup.md)** — Development environment setup
- **[02-first-contribution.md](onboarding/02-first-contribution.md)** — Adding a new game, contribution checklist
