import type React from "react";
import {
  ProgressBar as AriaProgressBar,
  type ProgressBarProps as AriaProgressBarProps,
  composeRenderProps,
} from "react-aria-components";
import { tv, type VariantProps } from "tailwind-variants";

const progressBarStyle = tv({
  slots: {
    root: "flex flex-col gap-1",
    track: "h-2 w-full overflow-hidden rounded-full bg-secondary",
    fill: "h-full transition-all duration-300",
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

export interface ProgressBarProps
  extends AriaProgressBarProps,
    ProgressBarStyleProps {
  className?: string;
  label?: string;
  showValue?: boolean;
  /** Custom fill color (overrides color variant) */
  fillColor?: string;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  className,
  label,
  showValue = false,
  color,
  size,
  fillColor,
  ...props
}) => {
  const styles = progressBarStyle({
    color: fillColor ? undefined : color,
    size,
  });

  return (
    <AriaProgressBar
      {...props}
      className={composeRenderProps(className, (cn) =>
        styles.root({ className: cn }),
      )}
    >
      {({ percentage = 0, valueText }) => (
        <>
          {(label || showValue) && (
            <div className="flex justify-between">
              {label && <span className={styles.label()}>{label}</span>}
              {showValue && (
                <span className={styles.valueText()}>{valueText}</span>
              )}
            </div>
          )}
          <div className={styles.track()}>
            <div
              className={styles.fill()}
              style={{
                width: `${percentage}%`,
                ...(fillColor && {
                  background: fillColor,
                  // Scale gradient to span full track width (clips based on fill %)
                  backgroundSize: `${10000 / Math.max(percentage, 1)}% 100%`,
                }),
              }}
            />
          </div>
        </>
      )}
    </AriaProgressBar>
  );
};
