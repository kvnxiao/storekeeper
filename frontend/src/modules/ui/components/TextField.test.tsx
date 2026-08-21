import { render, screen } from "@solidjs/testing-library";
import { describe, expect, it } from "vite-plus/test";
import { TextField } from "@/modules/ui/components/TextField";

describe("TextField", () => {
  it("applies a caller's class to the input", () => {
    render(() => <TextField label="Filter" value="" onChange={() => {}} inputClass="h-9" />);

    expect(screen.getByLabelText("Filter")).toHaveClass("h-9");
  });

  it("resolves a caller's class against the conflicting base utility", () => {
    render(() => <TextField label="Filter" value="" onChange={() => {}} inputClass="px-6" />);

    const input = screen.getByLabelText("Filter");
    expect(input).toHaveClass("px-6");
    expect(input).not.toHaveClass("px-3");
  });
});
