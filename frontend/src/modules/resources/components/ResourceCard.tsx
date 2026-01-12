import { motion, useReducedMotion } from "motion/react";

import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import { ExpeditionCard } from "@/modules/resources/components/ExpeditionCard";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import {
  isCooldownResource,
  isExpeditionResource,
  isStaminaResource,
} from "@/modules/resources/resources.types";
import {
  cardItemVariants,
  cardItemVariantsReduced,
} from "@/modules/ui/ui.animations";

interface ResourceCardProps {
  type: string;
  data: unknown;
}

export const ResourceCard: React.FC<ResourceCardProps> = ({ type, data }) => {
  const shouldReduceMotion = useReducedMotion();
  const variants = shouldReduceMotion
    ? cardItemVariantsReduced
    : cardItemVariants;

  return (
    <motion.div variants={variants}>
      {isStaminaResource(data) && <StaminaCard type={type} data={data} />}
      {isCooldownResource(data) && <CooldownCard type={type} data={data} />}
      {isExpeditionResource(data) && <ExpeditionCard type={type} data={data} />}
    </motion.div>
  );
};
