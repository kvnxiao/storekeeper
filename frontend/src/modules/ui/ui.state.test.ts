import { createRoot } from "solid-js";
import { describe, expect, it } from "vitest";
import { createUiState } from "@/modules/ui/ui.state";

function withState<T>(run: (state: ReturnType<typeof createUiState>) => T): T {
  return createRoot((dispose) => {
    const result = run(createUiState());
    dispose();
    return result;
  });
}

describe("ui state", () => {
  it("starts every section expanded", () => {
    expect(withState((state) => state.isSectionExpanded("genshin_impact"))).toBe(true);
  });

  it("remembers a collapsed section without touching its siblings", () => {
    withState((state) => {
      state.setSectionExpanded("genshin_impact", false);

      expect(state.isSectionExpanded("genshin_impact")).toBe(false);
      expect(state.isSectionExpanded("honkai_star_rail")).toBe(true);
    });
  });

  it("expands a section again", () => {
    withState((state) => {
      state.setSectionExpanded("genshin_impact", false);
      state.setSectionExpanded("genshin_impact", true);

      expect(state.isSectionExpanded("genshin_impact")).toBe(true);
    });
  });
});
