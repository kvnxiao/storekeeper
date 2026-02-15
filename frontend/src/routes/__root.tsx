import { type QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createRootRouteWithContext,
  HeadContent,
  Outlet,
  Scripts,
} from "@tanstack/react-router";
import { Provider as JotaiProvider } from "jotai";
import { useHydrateAtoms } from "jotai/utils";
import { queryClientAtom } from "jotai-tanstack-query";
import { queryClient } from "@/modules/core/core.queryClient";
import appCss from "@/styles.css?url";

interface RouterContext {
  queryClient: QueryClient;
}

/** Hydrates the shared QueryClient into jotai-tanstack-query synchronously */
const HydrateQueryClient: React.FC<React.PropsWithChildren> = ({
  children,
}) => {
  useHydrateAtoms([[queryClientAtom, queryClient]]);
  return <>{children}</>;
};

const RootComponent: React.FC = () => {
  return (
    <html lang="en">
      <head>
        <HeadContent />
      </head>
      <body className="min-h-screen overflow-y-scroll bg-background font-sans text-foreground antialiased">
        <QueryClientProvider client={queryClient}>
          <JotaiProvider>
            <HydrateQueryClient>
              <Outlet />
            </HydrateQueryClient>
          </JotaiProvider>
        </QueryClientProvider>
        <Scripts />
      </body>
    </html>
  );
};

export const Route = createRootRouteWithContext<RouterContext>()({
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1" },
      { title: "Storekeeper" },
    ],
    links: [{ rel: "stylesheet", href: appCss }],
  }),
  component: RootComponent,
});
