import { type QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import {
  createRootRouteWithContext,
  type ErrorComponentProps,
  HeadContent,
  Outlet,
  Scripts,
} from "@tanstack/solid-router";
import { type Component, createEffect, onMount, type ParentComponent, Show } from "solid-js";
import { HydrationScript } from "solid-js/web";
import { queryClient } from "@/modules/core/core.queryClient";
import { core } from "@/modules/core/core.state";
import { Button } from "@/modules/ui/components/Button";
import { ErrorBanner } from "@/modules/ui/components/ErrorBanner";
import * as m from "@/paraglide/messages";
import appCss from "@/styles.css?url";

interface RouterContext {
  queryClient: QueryClient;
}

const RootDocument: ParentComponent = (props) => (
  <html lang="en">
    <head>
      <HydrationScript />
    </head>
    <body class="min-h-screen overflow-y-scroll bg-background font-sans text-foreground antialiased">
      <HeadContent />
      {props.children}
      <Scripts />
    </body>
  </html>
);

const RootComponent: Component = () => {
  onMount(() => core.init());

  // The shell's static lang="en" never re-renders; keep it on the real locale.
  createEffect(() => {
    document.documentElement.lang = core.locale();
  });

  return (
    <QueryClientProvider client={queryClient}>
      {/* Nothing renders until the backend-resolved locale is in, so the first
          paint is never in the wrong language. Keyed on locale so a later
          change remounts the tree, since messages are not reactive. */}
      <Show when={core.localeReady() && core.locale()} keyed>
        <Outlet />
      </Show>
    </QueryClientProvider>
  );
};

const RootErrorPage: Component<ErrorComponentProps> = (props) => (
  <div class="flex min-h-screen flex-col items-center justify-center gap-4 p-4">
    <h1 class="text-lg font-bold text-zinc-950 dark:text-white">{m.error_title()}</h1>
    <ErrorBanner class="max-w-sm break-all font-mono text-sm">{String(props.error)}</ErrorBanner>
    <Button color="blue" onClick={() => window.location.reload()}>
      {m.error_reload()}
    </Button>
  </div>
);

export const Route = createRootRouteWithContext<RouterContext>()({
  head: () => ({
    meta: [
      { charset: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1" },
      { title: "Storekeeper" },
    ],
    links: [{ rel: "stylesheet", href: appCss }],
  }),
  shellComponent: RootDocument,
  component: RootComponent,
  errorComponent: RootErrorPage,
});
