import { render, screen } from "@solidjs/testing-library";
import { describe, expect, it } from "vite-plus/test";
import { Badge } from "./Badge";

describe("Badge", () => {
  it("renders children in a styled span", () => {
    render(() => <Badge variant="success">Claimed</Badge>);
    const badge = screen.getByText("Claimed");
    expect(badge.tagName).toBe("SPAN");
    expect(badge.className).toContain("inline-flex");
  });

  it("applies the default variant when none is given", () => {
    render(() => <Badge>Status</Badge>);
    expect(screen.getByText("Status").className).toContain("bg-zinc-600/10");
  });
});
