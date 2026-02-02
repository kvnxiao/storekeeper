import { useAtomValue } from "jotai";
import { motion, useReducedMotion } from "motion/react";

import { atoms } from "@/modules/atoms";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { isStaminaResource } from "@/modules/resources/resources.types";
import {
  cardItemVariants,
  cardItemVariantsReduced,
} from "@/modules/ui/ui.animations";

export const WaveplatesCard: React.FC = () => {
  const shouldReduceMotion = useReducedMotion();
  const variants = shouldReduceMotion
    ? cardItemVariantsReduced
    : cardItemVariants;

  const resource = useAtomValue(atoms.games.wuwa.waveplates);

  if (!resource || !isStaminaResource(resource.data)) return null;

  return (
    <motion.div variants={variants}>
      <StaminaCard type={resource.type} data={resource.data} />
    </motion.div>
  );
};
