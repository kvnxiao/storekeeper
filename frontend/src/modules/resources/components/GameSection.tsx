import { motion, useReducedMotion } from "motion/react";
import { useState } from "react";
import {
  Button,
  Disclosure,
  DisclosurePanel,
  Heading,
} from "react-aria-components";

import type { GameId } from "@/modules/games/games.types";
import { ResourceCard } from "@/modules/resources/components/ResourceCard";
import type { GameResource } from "@/modules/resources/resources.types";
import {
  cardContainerVariants,
  springTransition,
} from "@/modules/ui/ui.animations";

interface GameSectionProps {
  title: string;
  gameId: GameId;
  resources: GameResource[];
}

export const GameSection: React.FC<GameSectionProps> = ({
  title,
  gameId,
  resources,
}) => {
  const [isExpanded, setIsExpanded] = useState(true);
  const shouldReduceMotion = useReducedMotion();

  return (
    <Disclosure
      isExpanded={isExpanded}
      onExpandedChange={setIsExpanded}
      className="overflow-hidden rounded-lg bg-white shadow-sm ring-1 ring-zinc-950/5 dark:bg-zinc-800 dark:ring-white/10"
    >
      <Heading>
        <Button
          slot="trigger"
          className="flex w-full cursor-pointer items-center justify-between px-4 py-3 text-left transition-colors hover:bg-zinc-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring dark:hover:bg-zinc-700"
        >
          <span className="text-lg font-semibold text-zinc-950 dark:text-white">
            {title}
          </span>
          <motion.span
            className="text-zinc-400"
            animate={{ rotate: isExpanded ? 0 : -90 }}
            transition={shouldReduceMotion ? { duration: 0 } : springTransition}
          >
            ▼
          </motion.span>
        </Button>
      </Heading>
      <DisclosurePanel className="h-(--disclosure-panel-height) overflow-clip transition-[height] duration-250 ease-out motion-reduce:transition-none">
        <motion.div
          className="grid grid-cols-2 gap-3 px-4 pb-4"
          variants={cardContainerVariants}
          initial="hidden"
          animate={isExpanded ? "visible" : "hidden"}
        >
          {resources.map((resource, index) => (
            <ResourceCard
              key={`${gameId}-${resource.type}-${index}`}
              type={resource.type}
              data={resource.data}
            />
          ))}
        </motion.div>
      </DisclosurePanel>
    </Disclosure>
  );
};
