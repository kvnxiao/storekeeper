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
 * Updates automatically based on the tick atom and locale atom.
 */
export function useFormattedTime(
  datetime: string | null | undefined,
): FormattedTime {
  const tick = useAtomValue(atoms.core.tick);
  const locale = useAtomValue(atoms.core.locale);

  const relativeTime = useMemo(
    () => formatTimeRemaining(datetime, tick, locale),
    [datetime, tick, locale],
  );

  const absoluteTime = useMemo(
    () => formatAbsoluteDateTime(datetime, tick, locale),
    [datetime, tick, locale],
  );

  return { relativeTime, absoluteTime };
}
