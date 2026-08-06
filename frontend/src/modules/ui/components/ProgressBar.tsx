import * as ProgressPrimitive from "@kobalte/core/progress";
import type { VoidComponent } from "solid-js";
import { tv } from "tailwind-variants";

const progressBarStyle = tv({
  slots: {
    root: "flex flex-col gap-1",
    track: "h-1 w-full overflow-hidden rounded-full bg-secondary",
    fill: "h-full w-(--kb-progress-fill-width) bg-primary transition-all duration-300",
  },
});

const styles = progressBarStyle();

export interface ProgressBarProps {
  "aria-label"?: string;
  class?: string;
  /** Percent filled, on Kobalte's default 0-100 scale. */
  value: number;
  /** Custom fill color or gradient (replaces the default fill) */
  fillColor?: string;
}

export const ProgressBar: VoidComponent<ProgressBarProps> = (props) => {
  return (
    <ProgressPrimitive.Root
      value={props.value}
      aria-label={props["aria-label"]}
      class={styles.root({ class: props.class })}
    >
      <ProgressPrimitive.Track class={styles.track()}>
        <ProgressPrimitive.Fill
          class={styles.fill()}
          style={
            props.fillColor
              ? {
                  background: props.fillColor,
                  // Scale gradient to span full track width (clips based on fill %)
                  "background-size": `${10000 / Math.max(props.value, 1)}% 100%`,
                }
              : undefined
          }
        />
      </ProgressPrimitive.Track>
    </ProgressPrimitive.Root>
  );
};
