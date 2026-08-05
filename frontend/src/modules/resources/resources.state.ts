import { createRoot, createSignal } from "solid-js";

function createResourcesState() {
  const [isRefreshing, setIsRefreshing] = createSignal(false);

  function refreshStarted(): void {
    setIsRefreshing(true);
  }

  function refreshSettled(): void {
    setIsRefreshing(false);
  }

  return { isRefreshing, refreshStarted, refreshSettled };
}

/** Event-driven refresh state; written by the backend event listeners in `core.init()`. */
export const resourcesState = createRoot(createResourcesState);
