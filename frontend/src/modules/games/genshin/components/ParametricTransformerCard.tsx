import { useAtomValue } from "jotai";
import { motion, useReducedMotion } from "motion/react";

import { atoms } from "@/modules/atoms";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import { isCooldownResource } from "@/modules/resources/resources.types";
import {
  cardItemVariants,
  cardItemVariantsReduced,
} from "@/modules/ui/ui.animations";

export const ParametricTransformerCard: React.FC = () => {
  const shouldReduceMotion = useReducedMotion();
  const variants = shouldReduceMotion
    ? cardItemVariantsReduced
    : cardItemVariants;

  const resource = useAtomValue(atoms.games.genshin.parametricTransformer);

  if (!resource || !isCooldownResource(resource.data)) return null;

  return (
    <motion.div variants={variants}>
      <CooldownCard type={resource.type} data={resource.data} />
    </motion.div>
  );
};
