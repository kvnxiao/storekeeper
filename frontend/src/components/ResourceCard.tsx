import { motion, useReducedMotion } from "motion/react";

import { CooldownCard } from "@/components/app/CooldownCard";
import { ExpeditionCard } from "@/components/app/ExpeditionCard";
import { StaminaCard } from "@/components/app/StaminaCard";
import {
  cardItemVariants,
  cardItemVariantsReduced,
} from "@/components/ui/animations";
import type { GameId } from "@/types";
import {
  isCooldownResource,
  isExpeditionResource,
  isStaminaResource,
} from "@/types";

interface ResourceCardProps {
  gameId: GameId;
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
