import { QueryClientProvider } from "@tanstack/react-query";
import {
  createRootRouteWithContext,
  HeadContent,
  Outlet,
  Scripts,
} from "@tanstack/react-router";
import { Provider as JotaiProvider } from "jotai";
import { useHydrateAtoms } from "jotai/utils";
import { queryClientAtom } from "jotai-tanstack-query";

import { queryClient } from "@/router";
import appCss from "@/styles.css?url";

interface RouterContext {
  queryClient: typeof queryClient;
}

/** Hydrates the shared QueryClient into jotai-tanstack-query synchronously */
const HydrateQueryClient: React.FC<{ children: React.ReactNode }> = ({
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
      <body className="antialiased">
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
