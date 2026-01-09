import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { useState } from "react";
import { Disclosure, DisclosurePanel } from "react-aria-components";

import { ResourceCard } from "@/components/ResourceCard";
import {
  cardContainerVariants,
  panelVariants,
  panelVariantsReduced,
  springTransition,
} from "@/components/ui/animations";
import type { GameId, GameResource } from "@/types";

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
  const variants = shouldReduceMotion ? panelVariantsReduced : panelVariants;

  return (
    <Disclosure
      isExpanded={isExpanded}
      onExpandedChange={setIsExpanded}
      className="overflow-hidden rounded-lg bg-white shadow-sm dark:bg-gray-800"
    >
      <h2>
        <button
          type="button"
          slot="trigger"
          className="flex w-full cursor-pointer items-center justify-between px-4 py-3 text-left transition-colors hover:bg-gray-50 dark:hover:bg-gray-700"
          onClick={() => setIsExpanded((prev) => !prev)}
        >
          <span className="text-lg font-semibold text-gray-900 dark:text-white">
            {title}
          </span>
          <motion.span
            className="text-gray-400"
            animate={{ rotate: isExpanded ? 0 : -90 }}
            transition={shouldReduceMotion ? { duration: 0 } : springTransition}
          >
            ▼
          </motion.span>
        </button>
      </h2>
      <AnimatePresence initial={false}>
        {isExpanded && (
          <DisclosurePanel>
            <motion.div
              initial="collapsed"
              animate="expanded"
              exit="collapsed"
              variants={variants}
              transition={
                shouldReduceMotion ? { duration: 0.15 } : springTransition
              }
              style={{ overflow: "hidden" }}
            >
              <motion.div
                className="grid grid-cols-2 gap-3 px-4 pb-4"
                variants={cardContainerVariants}
                initial="hidden"
                animate="visible"
              >
                {resources.map((resource, index) => (
                  <ResourceCard
                    key={`${gameId}-${resource.type}-${index}`}
                    gameId={gameId}
                    type={resource.type}
                    data={resource.data}
                  />
                ))}
              </motion.div>
            </motion.div>
          </DisclosurePanel>
        )}
      </AnimatePresence>
    </Disclosure>
  );
};
