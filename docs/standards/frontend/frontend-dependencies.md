# Dependency Management

## Package Manager

Use [pnpm](https://pnpm.io/) (version specified in `package.json`):

```json
{
  "packageManager": "pnpm@10.27.0"
}
```

### Commands

```bash
pnpm install              # Install dependencies
pnpm add <package>        # Add runtime dependency
pnpm add -D <package>     # Add dev dependency
pnpm remove <package>     # Remove dependency
pnpm update               # Update all dependencies
```

## Dependency Categories

### dependencies

Runtime code bundled with the app:

```json
{
  "dependencies": {
    "react": "^19.2.3",
    "jotai": "^2.12.3",
    "motion": "^12.24.12"
  }
}
```

### devDependencies

Build and test tools only:

```json
{
  "devDependencies": {
    "@biomejs/biome": "^2.0.2",
    "typescript": "^5.8.2",
    "vitest": "^3.1.4"
  }
}
```

```tsx
// Bad: Test library in dependencies
"dependencies": { "@testing-library/react": "^16.3.1" }

// Good: Test library in devDependencies
"devDependencies": { "@testing-library/react": "^16.3.1" }
```

## Version Strategy

Use caret ranges (`^`) for application dependencies:

```json
"react": "^19.2.3"  // Allows 19.x.x updates
```

Pin exact versions only when required for stability, with documentation:

```json
// Pinned due to breaking change in 3.0.2
"nitro": "npm:nitro-nightly@3.0.1-20251230-165713-6e801e22"
```

## Adding New Dependencies

Before adding a package, evaluate:

1. **Necessity**: Can native browser/React APIs solve this?
2. **Bundle size**: Check [bundlephobia.com](https://bundlephobia.com/)
3. **Maintenance**: Is it actively maintained?
4. **Types**: Does it have TypeScript support?
5. **Security**: Any known vulnerabilities?

### Checklist Before Adding

- [ ] No native API alternative
- [ ] Bundle size acceptable
- [ ] Package actively maintained (commits in last 6 months)
- [ ] TypeScript types available
- [ ] No known security vulnerabilities

## Security Auditing

Run audits regularly:

```bash
pnpm audit              # Check for vulnerabilities
pnpm audit --fix        # Auto-fix where possible
pnpm outdated           # List outdated packages
```

Update vulnerable dependencies promptly.

## Lock File

Always commit `pnpm-lock.yaml`. In CI, use:

```bash
pnpm install --frozen-lockfile
```

This ensures reproducible builds.

## Checklist

- [ ] Test/build tools in devDependencies
- [ ] Lock file committed
- [ ] `pnpm audit` clean
- [ ] Pinned versions documented
- [ ] New dependencies pass evaluation criteria
