import type { Transition, Variants } from "motion/react";

/** Spring transition for natural feel */
export const springTransition: Transition = {
  type: "spring",
  stiffness: 300,
  damping: 30,
};

/** Disclosure panel expand/collapse variants */
export const panelVariants: Variants = {
  collapsed: { height: 0, opacity: 0 },
  expanded: { height: "auto", opacity: 1 },
};

/** Reduced motion panel variants (fade only) */
export const panelVariantsReduced: Variants = {
  collapsed: { opacity: 0 },
  expanded: { opacity: 1 },
};

/** Card container with stagger effect */
export const cardContainerVariants: Variants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { when: "beforeChildren", staggerChildren: 0.03 },
  },
};

/** Individual card item animation */
export const cardItemVariants: Variants = {
  hidden: { opacity: 0, y: 6 },
  visible: { opacity: 1, y: 0, transition: { duration: 0.15 } },
};

/** Reduced motion card variants (fade only) */
export const cardItemVariantsReduced: Variants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1 },
};
