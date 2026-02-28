import { invoke } from "@tauri-apps/api/core";
import { motion, useReducedMotion } from "motion/react";
import { useCallback, useState } from "react";
import {
  Button,
  Disclosure,
  DisclosurePanel,
  Heading,
} from "react-aria-components";
import { tv } from "tailwind-variants";
import type { GameId } from "@/modules/games/games.types";
import { Badge } from "@/modules/ui/components/Badge";
import {
  cardContainerVariants,
  springTransition,
} from "@/modules/ui/ui.animations";
import * as m from "@/paraglide/messages";

const disclosureStyle = tv({
  base: "overflow-hidden rounded-lg bg-white shadow-sm ring-1 ring-zinc-950/5 dark:bg-zinc-800 dark:ring-white/10",
});

const triggerStyle = tv({
  base: [
    "flex w-full cursor-pointer items-center justify-between px-3 py-2 text-left transition-colors",
    "hover:bg-zinc-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring",
    "dark:hover:bg-zinc-700",
  ],
});

interface GameSectionProps {
  title: string;
  /** Required when `claimStatus` is provided, to support manual claiming. */
  gameId?: GameId;
  claimStatus?: boolean | null;
  children: React.ReactNode;
}

export const GameSection: React.FC<GameSectionProps> = ({
  title,
  gameId,
  claimStatus,
  children,
}) => {
  const [isExpanded, setIsExpanded] = useState(true);
  const [isClaiming, setIsClaiming] = useState(false);
  const shouldReduceMotion = useReducedMotion();

  const handleClaim = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      if (isClaiming || gameId == null) {
        return;
      }
      setIsClaiming(true);
      invoke("claim_daily_reward_for_game", { gameId })
        .catch((err) => console.error("Failed to claim daily reward:", err))
        .finally(() => setIsClaiming(false));
    },
    [gameId, isClaiming],
  );

  return (
    <Disclosure
      isExpanded={isExpanded}
      onExpandedChange={setIsExpanded}
      className={disclosureStyle()}
    >
      <Heading>
        <Button slot="trigger" className={triggerStyle()}>
          <span className="flex items-center gap-2">
            <span className="text-base font-semibold text-zinc-950 dark:text-white">
              {title}
            </span>
            {claimStatus != null && (
              <ClaimBadge
                claimed={claimStatus}
                isClaiming={isClaiming}
                canClaim={gameId != null}
                onClaim={handleClaim}
              />
            )}
          </span>
          <motion.span
            className="text-zinc-400"
            animate={{ rotate: isExpanded ? 0 : -90 }}
            transition={shouldReduceMotion ? { duration: 0 } : springTransition}
          >
            ▼
          </motion.span>
        </Button>
      </Heading>
      <DisclosurePanel className="h-(--disclosure-panel-height) overflow-clip transition-[height] duration-250 ease-out motion-reduce:transition-none">
        <motion.div
          className="flex flex-col gap-1.5 px-2 pt-1.5 pb-2"
          variants={cardContainerVariants}
          initial={false}
          animate={isExpanded ? "visible" : "hidden"}
        >
          {children}
        </motion.div>
      </DisclosurePanel>
    </Disclosure>
  );
};

interface ClaimBadgeProps {
  claimed: boolean;
  isClaiming: boolean;
  canClaim: boolean;
  onClaim: (e: React.MouseEvent) => void;
}

const ClaimBadge: React.FC<ClaimBadgeProps> = ({
  claimed,
  isClaiming,
  canClaim,
  onClaim,
}) => {
  if (isClaiming) {
    return <Badge variant="default">{m.daily_claiming()}</Badge>;
  }

  if (claimed) {
    return <Badge variant="success">{m.daily_claimed()}</Badge>;
  }

  if (canClaim) {
    return (
      <Badge
        variant="warning"
        role="button"
        tabIndex={0}
        className="cursor-pointer"
        onClick={onClaim}
      >
        {m.daily_unclaimed()}
      </Badge>
    );
  }

  return <Badge variant="warning">{m.daily_unclaimed()}</Badge>;
};
