import * as ProgressPrimitive from "@kobalte/core/progress";
import { mergeProps, type VoidComponent } from "solid-js";
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
  value: number;
  minValue?: number;
  maxValue?: number;
  /** Custom fill color or gradient (replaces the default fill) */
  fillColor?: string;
}

export const ProgressBar: VoidComponent<ProgressBarProps> = (props) => {
  const merged = mergeProps({ minValue: 0, maxValue: 100 }, props);

  const percentage = () =>
    ((merged.value - merged.minValue) / (merged.maxValue - merged.minValue)) * 100;

  return (
    <ProgressPrimitive.Root
      value={merged.value}
      minValue={merged.minValue}
      maxValue={merged.maxValue}
      aria-label={merged["aria-label"]}
      class={styles.root({ class: merged.class })}
    >
      <ProgressPrimitive.Track class={styles.track()}>
        <ProgressPrimitive.Fill
          class={styles.fill()}
          style={
            merged.fillColor
              ? {
                  background: merged.fillColor,
                  // Scale gradient to span full track width (clips based on fill %)
                  "background-size": `${10000 / Math.max(percentage(), 1)}% 100%`,
                }
              : undefined
          }
        />
      </ProgressPrimitive.Track>
    </ProgressPrimitive.Root>
  );
};
