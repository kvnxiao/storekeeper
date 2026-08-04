import { type QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { createRootRouteWithContext, HeadContent, Outlet, Scripts } from "@tanstack/solid-router";
import { type Component, onMount, type ParentComponent, Show } from "solid-js";
import { HydrationScript } from "solid-js/web";
import { queryClient } from "@/modules/core/core.queryClient";
import { core } from "@/modules/core/core.state";
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

  return (
    <QueryClientProvider client={queryClient}>
      {/* Keyed on locale so the entire route tree remounts on locale change */}
      <Show when={core.locale()} keyed>
        <Outlet />
      </Show>
    </QueryClientProvider>
  );
};

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
});
