import { motion, useReducedMotion } from "motion/react";
import { useState } from "react";
import {
  Button,
  Disclosure,
  DisclosurePanel,
  Heading,
} from "react-aria-components";

import {
  cardContainerVariants,
  springTransition,
} from "@/modules/ui/ui.animations";

interface GameSectionWrapperProps {
  title: string;
  children: React.ReactNode;
}

export const GameSectionWrapper: React.FC<GameSectionWrapperProps> = ({
  title,
  children,
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
          className="flex w-full cursor-pointer items-center justify-between px-3 py-2 text-left transition-colors hover:bg-zinc-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring dark:hover:bg-zinc-700"
        >
          <span className="text-base font-semibold text-zinc-950 dark:text-white">
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
          className="flex flex-col gap-1.5 px-2 pb-2"
          variants={cardContainerVariants}
          initial="hidden"
          animate={isExpanded ? "visible" : "hidden"}
        >
          {children}
        </motion.div>
      </DisclosurePanel>
    </Disclosure>
  );
};
