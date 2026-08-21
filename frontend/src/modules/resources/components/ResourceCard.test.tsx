import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vite-plus/test";
import { GameId } from "@/modules/games/games.types";
import { ResourceCard } from "@/modules/resources/components/ResourceCard";

function renderCard(hasData: boolean): HTMLElement {
  const { container } = render(() => (
    <ResourceCard
      gameId={GameId.GenshinImpact}
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
  it("masks the card while it has no values to show", () => {
    const card = renderCard(false);

    expect(card).toHaveClass("mask-shimmer");
    expect(card).toHaveAttribute("data-shimmer", "active");
  });

  it("leaves a card with values unmasked", () => {
    const card = renderCard(true);

    expect(card).not.toHaveClass("mask-shimmer");
    expect(card).not.toHaveAttribute("data-shimmer");
  });
});
