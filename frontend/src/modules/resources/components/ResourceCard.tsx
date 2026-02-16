import type { Atom } from "jotai";
import { useAtomValue } from "jotai";
import { motion, useReducedMotion } from "motion/react";
import { atoms } from "@/modules/atoms";
import { getResourceDisplayName } from "@/modules/games/games.constants";
import { CooldownCard } from "@/modules/resources/components/CooldownCard";
import { StaminaCard } from "@/modules/resources/components/StaminaCard";
import type { GameResource } from "@/modules/resources/resources.types";
import {
  isCooldownResource,
  isStaminaResource,
} from "@/modules/resources/resources.types";
import {
  cardItemVariants,
  cardItemVariantsReduced,
} from "@/modules/ui/ui.animations";

interface ResourceCardProps {
  resourceAtom: Atom<GameResource | null>;
  resourceType: string;
  iconPath: string;
  variant: "stamina" | "cooldown";
}

export const ResourceCard: React.FC<ResourceCardProps> = ({
  resourceAtom,
  resourceType,
  iconPath,
  variant,
}) => {
  const shouldReduceMotion = useReducedMotion();
  const variants = shouldReduceMotion
    ? cardItemVariantsReduced
    : cardItemVariants;

  const resource = useAtomValue(resourceAtom);
  const isRefreshing = useAtomValue(atoms.core.isRefreshing);
  const resourceName = getResourceDisplayName(resourceType);

  if (variant === "stamina") {
    const data =
      resource && isStaminaResource(resource.data) ? resource.data : undefined;
    return (
      <motion.div variants={variants}>
        <StaminaCard
          iconPath={iconPath}
          name={resourceName}
          data={data}
          isRefreshing={isRefreshing}
        />
      </motion.div>
    );
  }

  const data =
    resource && isCooldownResource(resource.data) ? resource.data : undefined;
  return (
    <motion.div variants={variants}>
      <CooldownCard
        iconPath={iconPath}
        name={resourceName}
        data={data}
        isRefreshing={isRefreshing}
      />
    </motion.div>
  );
};
