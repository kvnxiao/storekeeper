import { useAtomValue } from "jotai";
import { useMemo } from "react";
import { atoms } from "@/modules/atoms";
import {
  formatAbsoluteDateTime,
  formatTimeRemaining,
} from "@/modules/resources/resources.utils";

interface FormattedTime {
  relativeTime: string;
  absoluteTime: string | null;
}

/**
 * Hook to format a datetime into relative and absolute time strings.
 * Updates automatically based on the tick atom.
 */
export function useFormattedTime(
  datetime: string | null | undefined,
): FormattedTime {
  const tick = useAtomValue(atoms.core.tick);

  const relativeTime = useMemo(
    () => formatTimeRemaining(datetime, tick),
    [datetime, tick],
  );

  const absoluteTime = useMemo(
    () => formatAbsoluteDateTime(datetime),
    [datetime],
  );

  return { relativeTime, absoluteTime };
}
