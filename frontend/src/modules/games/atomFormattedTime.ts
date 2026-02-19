import { atom, type Getter } from "jotai";
import type { CoreAtoms } from "@/modules/core/core.atoms";
import type { FormattedTime } from "@/modules/resources/resources.types";
import {
  formatAbsoluteDateTime,
  formatTimeRemaining,
} from "@/modules/resources/resources.utils";

/**
 * Creates a derived atom that computes formatted time for a resource.
 *
 * Re-evaluates on tick, locale change, or when the source datetime changes.
 */
export function atomFormattedTime(
  getCore: () => CoreAtoms,
  getDatetime: (get: Getter) => string | null | undefined,
) {
  return atom<FormattedTime>((get) => {
    const core = getCore();
    const nowMs = get(core.tick);
    const datetime = getDatetime(get);
    const durationFmt = get(core.durationFormatter);
    const timeOnlyFmt = get(core.timeOnlyFormatter);
    const weekdayTimeFmt = get(core.weekdayTimeFormatter);

    return {
      relativeTime: formatTimeRemaining(datetime, nowMs, durationFmt),
      absoluteTime: formatAbsoluteDateTime(
        datetime,
        nowMs,
        timeOnlyFmt,
        weekdayTimeFmt,
      ),
    };
  });
}
