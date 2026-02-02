import {
  Tooltip as AriaTooltip,
  type TooltipProps as AriaTooltipProps,
  composeRenderProps,
  OverlayArrow,
} from "react-aria-components";
import { tv } from "tailwind-variants";

const tooltipStyle = tv({
  base: "rounded bg-zinc-900 px-2 py-1 text-xs text-white shadow-md dark:bg-zinc-100 dark:text-zinc-900",
});

export interface TooltipProps extends AriaTooltipProps {}

export const Tooltip: React.FC<TooltipProps> = ({ className, ...props }) => {
  return (
    <AriaTooltip
      offset={8}
      {...props}
      className={composeRenderProps(className, (className) =>
        tooltipStyle({ className }),
      )}
    >
      {composeRenderProps(props.children, (children) => (
        <>
          <OverlayArrow>
            <svg
              aria-hidden="true"
              width={8}
              height={8}
              viewBox="0 0 8 8"
              className="block fill-zinc-900 dark:fill-zinc-100 group-placement-bottom:rotate-180 group-placement-left:-rotate-90 group-placement-right:rotate-90"
            >
              <path d="M0 0 L4 4 L8 0" />
            </svg>
          </OverlayArrow>
          {children}
        </>
      ))}
    </AriaTooltip>
  );
};
