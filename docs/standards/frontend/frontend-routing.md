# Routing Standards

## Overview

Routing uses [TanStack Router](https://tanstack.com/router) with file-based routes.

## File-Based Routes

Routes live in `src/routes/` with auto-generated route tree:

```
src/routes/
├── __root.tsx        # Root layout
├── index.tsx         # Home page (/)
└── settings.tsx      # Settings page (/settings)
```

### Route Definition

```tsx
// routes/settings.tsx
import { createFileRoute } from "@tanstack/react-router";

const SettingsPage: React.FC = () => {
  return <div>Settings</div>;
};

export const Route = createFileRoute("/settings")({
  component: SettingsPage,
});
```

**Note**: `src/routeTree.gen.ts` is auto-generated. Never edit manually.

## Router Context

Pass shared dependencies through router context:

```tsx
// router.tsx
import { queryClient } from "@/modules/core/core.queryClient";

interface RouterContext {
  queryClient: QueryClient;
}

export const router = createRouter({
  routeTree,
  context: { queryClient },
  scrollRestoration: true,
  defaultViewTransition: true,
});
```

### Root Route Setup

```tsx
// routes/__root.tsx
import { queryClient } from "@/modules/core/core.queryClient";

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
  head: () => ({
    meta: [{ charSet: "utf-8" }, { name: "viewport", content: "..." }],
    links: [{ rel: "stylesheet", href: "/src/styles.css" }],
  }),
});

const RootComponent: React.FC = () => (
  <html lang="en">
    <head>
      <HeadContent />
    </head>
    <body>
      <QueryClientProvider client={queryClient}>
        <JotaiProvider>
          <HydrateQueryClient>
            <Outlet />
          </HydrateQueryClient>
        </JotaiProvider>
      </QueryClientProvider>
    </body>
  </html>
);
```

## View Transitions

Enable iOS-style page transitions:

```tsx
// Router config
const router = createRouter({
  defaultViewTransition: true,
});
```

### Transition Direction

Set direction before navigation:

```tsx
import { ButtonLink } from "@/modules/ui/components/ButtonLink";

// Bad: No direction
<Link to="/settings">Settings</Link>

// Good: Set transition direction
<ButtonLink
  to="/settings"
  onClick={() => {
    document.documentElement.dataset.viewTransitionDirection = "forward";
  }}
>
  Settings
</ButtonLink>
```

Values: `"forward"` for push, `"back"` for pop.

## Router-Integrated Components

Use `createLink` to compose React Aria with TanStack Router:

```tsx
// modules/ui/components/Link.tsx
import { createLink } from "@tanstack/react-router";
import { Link as RACLink } from "react-aria-components";

export const Link = createLink(RACLink);

// Usage
import { Link } from "@/modules/ui/components/Link";
<Link to="/settings">Settings</Link>
```

### ButtonLink Pattern

```tsx
// modules/ui/components/ButtonLink.tsx
import { createLink, type LinkComponent } from "@tanstack/react-router";
import { Link as RACLink, type LinkProps as RACLinkProps } from "react-aria-components";
import { buttonStyle, type ButtonStyleProps } from "@/modules/ui/components/Button";

const ButtonLinkBase: React.FC<RACLinkProps & ButtonStyleProps> = ({
  variant,
  color,
  size,
  className,
  children,
  ...props
}) => (
  <RACLink
    {...props}
    className={composeRenderProps(className, (cn) =>
      buttonStyle({ variant, color, size, className: cn }),
    )}
  >
    {children}
  </RACLink>
);

export const ButtonLink: LinkComponent<typeof ButtonLinkBase> = createLink(ButtonLinkBase);

// Usage
import { ButtonLink } from "@/modules/ui/components/ButtonLink";
<ButtonLink to="/" variant="plain">Back</ButtonLink>
```

## Checklist

- [ ] Routes use `createFileRoute`
- [ ] Root uses `createRootRouteWithContext`
- [ ] QueryClient passed through router context
- [ ] View transitions enabled
- [ ] Navigation sets transition direction
- [ ] Link components use `createLink`
- [ ] Imports use `@/modules/*` paths
