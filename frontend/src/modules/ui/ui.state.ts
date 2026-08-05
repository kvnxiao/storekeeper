import { createRoot } from "solid-js";
import { createStore } from "solid-js/store";

/**
 * View state that has to outlive a route change.
 *
 * Navigating to settings unmounts the dashboard, so anything held in its
 * components resets on the way back: sections the user collapsed would spring
 * open again.
 */
export function createUiState() {
  const [collapsedSections, setCollapsedSections] = createStore<Record<string, boolean>>({});

  /** Sections start expanded; only an explicit collapse is remembered. */
  const isSectionExpanded = (sectionId: string): boolean => !collapsedSections[sectionId];

  function setSectionExpanded(sectionId: string, expanded: boolean): void {
    setCollapsedSections(sectionId, !expanded);
  }

  return { isSectionExpanded, setSectionExpanded };
}

export const uiState = createRoot(createUiState);
