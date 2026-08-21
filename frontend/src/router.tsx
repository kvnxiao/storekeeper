import "temporal-polyfill/global";

import { createRouter } from "@tanstack/solid-router";
import { queryClient } from "@/modules/core/core.queryClient";
import { routeTree } from "@/routeTree.gen";

export function getRouter() {
  return createRouter({
    routeTree,
    context: {
      queryClient,
    },
    scrollRestoration: true,
    defaultPreloadStaleTime: 0,
    defaultViewTransition: true,
  });
}

declare module "@tanstack/solid-router" {
  interface Register {
    router: ReturnType<typeof getRouter>;
  }
}
