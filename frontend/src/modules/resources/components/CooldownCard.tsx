import { motion, useReducedMotion } from "motion/react";
import { ResourceIcon } from "@/modules/resources/components/ResourceIcon";
import { TimeRemaining } from "@/modules/resources/components/TimeRemaining";
import type {
  CooldownResource,
  FormattedTime,
} from "@/modules/resources/resources.types";
import { Badge } from "@/modules/ui/components/Badge";
import {
  cardItemVariants,
  cardItemVariantsReduced,
} from "@/modules/ui/ui.animations";
import { cn } from "@/modules/ui/ui.styles";
import * as m from "@/paraglide/messages";

interface CooldownCardProps {
  iconPath: string;
  name: string;
  data?: CooldownResource;
  formattedTime: FormattedTime;
  isRefreshing?: boolean;
}

export const CooldownCard: React.FC<CooldownCardProps> = ({
  iconPath,
  name,
  data,
  formattedTime,
  isRefreshing,
}) => {
  const shouldReduceMotion = useReducedMotion();
  const variants = shouldReduceMotion
    ? cardItemVariantsReduced
    : cardItemVariants;

  // Loading state - show icon + name with shimmer badge placeholder
  if (!data) {
    return (
      <motion.div variants={variants}>
        <div className="mask-shimmer rounded-lg bg-zinc-50 p-2 transition-transform hover:translate-x-0.5 dark:bg-zinc-700">
          <div className="flex items-center gap-2">
            <ResourceIcon src={iconPath} size="md" />
            <span className="min-w-0 flex-1 truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
              {name}
            </span>
            <div className="h-5 w-14 rounded-full bg-zinc-200 dark:bg-zinc-600" />
          </div>
          {/* h-4 matches text-xs line-height (1rem = 16px) */}
          <div className="mt-1 h-4 w-24 rounded bg-zinc-200 dark:bg-zinc-600" />
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div variants={variants}>
      <div
        className={cn(
          "rounded-lg bg-zinc-50 p-2 transition-transform hover:translate-x-0.5 dark:bg-zinc-700",
          isRefreshing && "mask-shimmer",
        )}
      >
        <div className="flex items-center gap-2">
          <ResourceIcon src={iconPath} size="md" />
          <span className="min-w-0 flex-1 truncate text-sm font-medium text-zinc-700 dark:text-zinc-300">
            {name}
          </span>
          {data.isReady ? (
            <Badge variant="success">{m.cooldown_ready()}</Badge>
          ) : (
            <Badge variant="warning">
              <TimeRemaining
                relativeTime={formattedTime.relativeTime}
                absoluteTime={formattedTime.absoluteTime}
                plain
              />
            </Badge>
          )}
        </div>
        <div className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
          {data.isReady ? (
            m.cooldown_ready()
          ) : (
            <>
              {m.cooldown_ready_in()}{" "}
              <TimeRemaining
                relativeTime={formattedTime.relativeTime}
                absoluteTime={formattedTime.absoluteTime}
              />
            </>
          )}
        </div>
      </div>
    </motion.div>
  );
};
