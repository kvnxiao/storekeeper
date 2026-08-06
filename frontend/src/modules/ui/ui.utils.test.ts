import { describe, expect, it } from "vite-plus/test";
import { setViewTransitionDirection } from "@/modules/ui/ui.utils";

describe("setViewTransitionDirection", () => {
  it("exposes the direction as the data attribute the stylesheet selects on", () => {
    setViewTransitionDirection("forward");
    expect(document.documentElement.dataset.viewTransitionDirection).toBe("forward");

    setViewTransitionDirection("back");
    expect(document.documentElement.getAttribute("data-view-transition-direction")).toBe("back");
  });
});
