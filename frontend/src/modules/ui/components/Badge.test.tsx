import { render, screen } from "@solidjs/testing-library";
import { describe, expect, it } from "vite-plus/test";
import { Badge } from "@/modules/ui/components/Badge";

describe("Badge", () => {
  it("renders children in a span", () => {
    render(() => <Badge variant="success">Claimed</Badge>);
    expect(screen.getByText("Claimed").tagName).toBe("SPAN");
  });

  it("styles variants differently from the default", () => {
    render(() => (
      <>
        <Badge>Status</Badge>
        <Badge variant="success">Claimed</Badge>
      </>
    ));

    expect(screen.getByText("Status").className).not.toBe(screen.getByText("Claimed").className);
  });
});
