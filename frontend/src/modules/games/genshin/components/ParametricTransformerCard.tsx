import { useAtomValue } from "jotai";
import { motion, useReducedMotion } from "motion/react";
import { atoms } from "@/modules/atoms";
import { RESOURCE_DISPLAY_NAMES } from "@/modules/games/games.constants";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import { isCooldownResource } from "@/modules/resources/resources.types";
import {
  cardItemVariants,
  cardItemVariantsReduced,
} from "@/modules/ui/ui.animations";

const RESOURCE_NAME = RESOURCE_DISPLAY_NAMES.parametric_transformer;
const RESOURCE_ICON = "/icons/game/genshin/Item_Parametric_Transformer.webp";

export const ParametricTransformerCard: React.FC = () => {
  const shouldReduceMotion = useReducedMotion();
  const variants = shouldReduceMotion
    ? cardItemVariantsReduced
    : cardItemVariants;

  const resource = useAtomValue(atoms.games.genshin.parametricTransformer);
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);

  const data =
    resource && isCooldownResource(resource.data) ? resource.data : undefined;

  return (
    <motion.div variants={variants}>
      <CooldownCard
        iconPath={RESOURCE_ICON}
        name={RESOURCE_NAME}
        data={data}
        isRefreshing={isRefreshing}
      />
    </motion.div>
  );
};
