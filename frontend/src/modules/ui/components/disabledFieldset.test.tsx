import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vite-plus/test";
import { NumberField } from "@/modules/ui/components/NumberField";
import { Switch } from "@/modules/ui/components/Switch";

/**
 * The settings route blocks edits during a save with a disabled <fieldset>,
 * which sets no Kobalte `disabled` prop and so emits no `data-disabled`.
 * Controls must key their disabled styling off the native state instead.
 */
function renderInDisabledFieldset(): HTMLElement {
  const { container } = render(() => (
    <fieldset disabled>
      <Switch checked={false} onChange={() => {}}>
        toggle
      </Switch>
      <NumberField value={1} onChange={() => {}} minValue={0} maxValue={10} step={1} />
    </fieldset>
  ));
  return container;
}

describe("controls inside a disabled fieldset", () => {
  it("leaves every input matching :disabled", () => {
    const inputs = [...renderInDisabledFieldset().querySelectorAll("input")];

    expect(inputs.length).toBeGreaterThan(0);
    expect(inputs.filter((input) => !input.matches(":disabled"))).toEqual([]);
  });

  it("carries no data-disabled for styling to hang off", () => {
    expect(renderInDisabledFieldset().querySelector("[data-disabled]")).toBeNull();
  });

  it("styles both controls on the native state", () => {
    const html = renderInDisabledFieldset().innerHTML;

    expect(html).toContain("group-has-[:disabled]:bg-zinc-100");
    expect(html).toContain("has-[:disabled]:bg-zinc-100");
    expect(html).not.toContain("data-[disabled]:");
  });

  // Without the gate the interactive backgrounds would still match while
  // disabled, and the disabled one would need !important to outrank them.
  it("gates the switch's interactive backgrounds rather than forcing them", () => {
    const track = renderInDisabledFieldset().querySelector("input.peer")?.nextElementSibling;
    const classes = [...(track?.classList ?? [])];

    expect(classes).toContain("group-not-has-[:disabled]:group-hover:bg-zinc-300");
    expect(classes.filter((name) => name.endsWith("!"))).toEqual([]);
  });
});
