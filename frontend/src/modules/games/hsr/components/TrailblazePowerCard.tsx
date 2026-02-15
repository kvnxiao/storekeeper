import { useAtomValue } from "jotai";
import { motion, useReducedMotion } from "motion/react";
import { atoms } from "@/modules/atoms";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import { isStaminaResource } from "@/modules/resources/resources.types";
import {
  cardItemVariants,
  cardItemVariantsReduced,
} from "@/modules/ui/ui.animations";

const RESOURCE_NAME = "Trailblaze Power";
const RESOURCE_ICON = "/icons/game/hsr/Item_Trailblaze_Power.webp";

export const TrailblazePowerCard: React.FC = () => {
  const shouldReduceMotion = useReducedMotion();
  const variants = shouldReduceMotion
    ? cardItemVariantsReduced
    : cardItemVariants;

  const resource = useAtomValue(atoms.games.hsr.trailblazePower);
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);

  const data =
    resource && isStaminaResource(resource.data) ? resource.data : undefined;

  return (
    <motion.div variants={variants}>
      <StaminaCard
        iconPath={RESOURCE_ICON}
        name={RESOURCE_NAME}
        data={data}
        isRefreshing={isRefreshing}
      />
    </motion.div>
  );
};
