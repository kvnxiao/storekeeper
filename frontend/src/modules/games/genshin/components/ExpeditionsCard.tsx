import { useAtomValue } from "jotai";
import { motion, useReducedMotion } from "motion/react";
import { atoms } from "@/modules/atoms";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import { isExpeditionResource } from "@/modules/resources/resources.types";
import {
  cardItemVariants,
  cardItemVariantsReduced,
} from "@/modules/ui/ui.animations";

const RESOURCE_NAME = "Expeditions";
const RESOURCE_ICON = "/icons/game/genshin/Expeditions.webp";

export const ExpeditionsCard: React.FC = () => {
  const shouldReduceMotion = useReducedMotion();
  const variants = shouldReduceMotion
    ? cardItemVariantsReduced
    : cardItemVariants;

  const resource = useAtomValue(atoms.games.genshin.expeditions);
  const allDone = useAtomValue(atoms.games.genshin.expeditionsReady);
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);

  const data =
    resource && isExpeditionResource(resource.data)
      ? { isReady: allDone, readyAt: resource.data.earliestFinishAt }
      : undefined;

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
