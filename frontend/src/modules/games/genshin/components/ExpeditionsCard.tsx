import { useAtomValue } from "jotai";
import { motion, useReducedMotion } from "motion/react";

import { atoms } from "@/modules/atoms";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import { isExpeditionResource } from "@/modules/resources/resources.types";
import {
  cardItemVariants,
  cardItemVariantsReduced,
} from "@/modules/ui/ui.animations";

export const ExpeditionsCard: React.FC = () => {
  const shouldReduceMotion = useReducedMotion();
  const variants = shouldReduceMotion
    ? cardItemVariantsReduced
    : cardItemVariants;

  const resource = useAtomValue(atoms.games.genshin.expeditions);
  const allDone = useAtomValue(atoms.games.genshin.expeditionsReady);

  if (!resource || !isExpeditionResource(resource.data)) return null;

  return (
    <motion.div variants={variants}>
      <CooldownCard
        type={resource.type}
        data={{
          isReady: allDone,
          readyAt: resource.data.earliestFinishAt,
        }}
      />
    </motion.div>
  );
};
