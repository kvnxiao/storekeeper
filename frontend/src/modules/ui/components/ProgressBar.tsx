import * as ProgressPrimitive from "@kobalte/core/progress";
import { mergeProps, Show, type VoidComponent } from "solid-js";
import { tv, type VariantProps } from "tailwind-variants";

const progressBarStyle = tv({
  slots: {
    root: "flex flex-col gap-1",
    track: "h-2 w-full overflow-hidden rounded-full bg-secondary",
    fill: "h-full w-(--kb-progress-fill-width) transition-all duration-300",
    label: "text-sm font-medium",
    valueText: "text-sm text-muted-foreground",
  },
  variants: {
    color: {
      default: { fill: "bg-primary" },
      success: { fill: "bg-green-500" },
      warning: { fill: "bg-amber-500" },
      danger: { fill: "bg-red-500" },
      info: { fill: "bg-blue-500" },
    },
    size: {
      xs: { track: "h-1" },
      sm: { track: "h-1.5" },
      md: { track: "h-2" },
      lg: { track: "h-3" },
    },
  },
  defaultVariants: {
    color: "default",
    size: "md",
  },
});

type ProgressBarStyleProps = VariantProps<typeof progressBarStyle>;

export interface ProgressBarProps extends ProgressBarStyleProps {
  "aria-label"?: string;
  class?: string;
  label?: string;
  showValue?: boolean;
  value: number;
  minValue?: number;
  maxValue?: number;
  /** Custom fill color (overrides color variant) */
  fillColor?: string;
}

export const ProgressBar: VoidComponent<ProgressBarProps> = (props) => {
  const merged = mergeProps({ minValue: 0, maxValue: 100, showValue: false }, props);

  const styles = () =>
    progressBarStyle({
      color: merged.fillColor ? undefined : merged.color,
      size: merged.size,
    });

  const percentage = () =>
    ((merged.value - merged.minValue) / (merged.maxValue - merged.minValue)) * 100;

  return (
    <ProgressPrimitive.Root
      value={merged.value}
      minValue={merged.minValue}
      maxValue={merged.maxValue}
      aria-label={merged["aria-label"]}
      class={styles().root({ class: merged.class })}
    >
      <Show when={merged.label || merged.showValue}>
        <div class="flex justify-between">
          <Show when={merged.label}>
            <ProgressPrimitive.Label class={styles().label()}>
              {merged.label}
            </ProgressPrimitive.Label>
          </Show>
          <Show when={merged.showValue}>
            <ProgressPrimitive.ValueLabel class={styles().valueText()} />
          </Show>
        </div>
      </Show>
      <ProgressPrimitive.Track class={styles().track()}>
        <ProgressPrimitive.Fill
          class={styles().fill()}
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
