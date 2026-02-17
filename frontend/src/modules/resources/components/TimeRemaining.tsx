import { Button, TooltipTrigger } from "react-aria-components";
import { tv } from "tailwind-variants";
import { Tooltip } from "@/modules/ui/components/Tooltip";
import { cn } from "@/modules/ui/ui.styles";

interface TimeRemainingProps {
  relativeTime: string;
  absoluteTime: string | null;
  className?: string;
  /** Skip background styling (use when nested inside Badge) */
  plain?: boolean;
}

const interactiveStyle = tv({
  base: [
    "rounded bg-zinc-100 px-1 text-zinc-700 transition-colors",
    "hover:bg-zinc-200 dark:bg-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-600",
  ],
});

export const TimeRemaining: React.FC<TimeRemainingProps> = ({
  relativeTime,
  absoluteTime,
  className,
  plain = false,
}) => {
  if (!absoluteTime) {
    return <time className={className}>{relativeTime}</time>;
  }

  return (
    <TooltipTrigger delay={300}>
      <Button className={cn(!plain && interactiveStyle(), className)}>
        {relativeTime}
      </Button>
      <Tooltip>{absoluteTime}</Tooltip>
    </TooltipTrigger>
  );
};
