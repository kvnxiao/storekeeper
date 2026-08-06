import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vite-plus/test";
import { Button } from "@/modules/ui/components/Button";
import { NumberField } from "@/modules/ui/components/NumberField";
import { SegmentedControl } from "@/modules/ui/components/SegmentedControl";
import { Select } from "@/modules/ui/components/Select";
import { Switch } from "@/modules/ui/components/Switch";
import { TextField } from "@/modules/ui/components/TextField";
import { Tooltip } from "@/modules/ui/components/Tooltip";

/**
 * The settings route wraps its fields in a <form>, where a <button> with no
 * explicit type submits. Every control below sits inside that form, so an
 * implicit submit would save the whole config on a stray click. Button is
 * included because its safety comes from a Kobalte default rather than from
 * anything in this repo.
 */
describe("controls inside the settings form", () => {
  it("never leaves a button that implicitly submits", () => {
    const { container } = render(() => (
      <form>
        <Tooltip content="tip">indicator</Tooltip>
        <Button onClick={() => {}}>undo</Button>
        <TextField type="password" value="secret" onChange={() => {}} />
        <NumberField value={1} onChange={() => {}} minValue={0} maxValue={10} step={1} />
        <Select
          value="a"
          onChange={() => {}}
          options={[
            { id: "a", label: "A" },
            { id: "b", label: "B" },
          ]}
        />
        <SegmentedControl
          aria-label="mode"
          selectedKey="a"
          onSelectionChange={() => {}}
          items={[
            { id: "a", label: "A" },
            { id: "b", label: "B" },
          ]}
        />
        <Switch checked={false} onChange={() => {}}>
          toggle
        </Switch>
      </form>
    ));

    const submitters = [...container.querySelectorAll("button")].filter(
      (button) => button.type === "submit",
    );

    expect(submitters.map((button) => button.outerHTML)).toEqual([]);
  });
});
