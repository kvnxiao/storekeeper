import { cleanup, render } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { ResourceCard } from "./ResourceCard";

function renderCard(hasData: boolean): HTMLElement {
  const { container } = render(() => (
    <ResourceCard
      iconPath="/icons/game/genshin/Item_Original_Resin.webp"
      name="Original Resin"
      hasData={hasData}
      trailing={<span>100/200</span>}
    />
  ));
  const card = container.firstElementChild;
  if (!(card instanceof HTMLElement)) {
    throw new Error("card did not render");
  }
  return card;
}

describe("ResourceCard", () => {
  afterEach(cleanup);

  it("masks the card while it has no values to show", () => {
    expect(renderCard(false).classList.contains("mask-shimmer")).toBe(true);
  });

  it("leaves a card with values unmasked", () => {
    expect(renderCard(true).classList.contains("mask-shimmer")).toBe(false);
  });
});
