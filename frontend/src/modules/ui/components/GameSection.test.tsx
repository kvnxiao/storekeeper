import { cleanup, render } from "@solidjs/testing-library";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vite-plus/test";
import { GameSection } from "@/modules/ui/components/GameSection";

function renderSection(): { root: HTMLElement; trigger: HTMLElement } {
  const view = render(() => (
    <GameSection sectionId="genshin_impact" title="Genshin Impact">
      <div>Original Resin</div>
    </GameSection>
  ));
  const root = view.container.firstElementChild;
  if (!(root instanceof HTMLElement)) {
    throw new Error("section did not render");
  }
  return { root, trigger: view.getByRole("button", { name: "Genshin Impact" }) };
}

describe("GameSection", () => {
  it("starts expanded", () => {
    expect(renderSection().root).toHaveAttribute("data-expanded", "");
  });

  it("keeps a collapsed section collapsed across a remount", async () => {
    const { root, trigger } = renderSection();
    await userEvent.setup().click(trigger);
    expect(root).toHaveAttribute("data-closed", "");

    cleanup();

    expect(renderSection().root).toHaveAttribute("data-closed", "");
  });
});
