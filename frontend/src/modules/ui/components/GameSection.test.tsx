import { cleanup, render } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { GameSection } from "./GameSection";

function renderSection(): { root: HTMLElement; trigger: HTMLElement } {
  const { container } = render(() => (
    <GameSection sectionId="genshin_impact" title="Genshin Impact">
      <div>Original Resin</div>
    </GameSection>
  ));
  const root = container.firstElementChild;
  const trigger = container.querySelector("button");
  if (!(root instanceof HTMLElement) || !(trigger instanceof HTMLElement)) {
    throw new Error("section did not render");
  }
  return { root, trigger };
}

describe("GameSection", () => {
  afterEach(cleanup);

  it("starts expanded", () => {
    expect(renderSection().root.dataset.expanded).toBe("");
  });

  it("keeps a collapsed section collapsed across a remount", () => {
    const { root, trigger } = renderSection();
    trigger.click();
    expect(root.dataset.closed).toBe("");

    cleanup();

    expect(renderSection().root.dataset.closed).toBe("");
  });
});
