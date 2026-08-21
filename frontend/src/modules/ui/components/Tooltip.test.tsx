import { render, screen } from "@solidjs/testing-library";
import { describe, expect, it } from "vite-plus/test";
import { TooltipButton } from "@/modules/ui/components/Tooltip";

describe("TooltipButton", () => {
  it("renders the trigger as the only button", () => {
    render(() => (
      <TooltipButton
        tooltip="Preview Waveplates notification"
        size="icon"
        variant="plain"
        aria-label="Preview Waveplates notification"
      />
    ));

    expect(screen.getAllByRole("button")).toHaveLength(1);
  });

  it("keeps the trigger out of a form submit", () => {
    render(() => <TooltipButton tooltip="Preview" aria-label="Preview" />);

    expect(screen.getByRole("button")).toHaveAttribute("type", "button");
  });
});
