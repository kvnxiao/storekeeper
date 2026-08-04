import { type Accessor, createMemo } from "solid-js";
import { core } from "@/modules/core/core.state";
import type { FormattedTime } from "@/modules/resources/resources.types";
import { formatAbsoluteDateTime, formatTimeRemaining } from "@/modules/resources/resources.utils";

/**
 * Derives formatted time for a resource datetime.
 *
 * Re-evaluates on tick, locale change, or when the source datetime changes.
 */
export function createFormattedTime(
  datetime: Accessor<string | null | undefined>,
): Accessor<FormattedTime> {
  return createMemo(() => ({
    relativeTime: formatTimeRemaining(datetime(), core.tick(), core.durationFormatter()),
    absoluteTime: formatAbsoluteDateTime(
      datetime(),
      core.tick(),
      core.timeOnlyFormatter(),
      core.weekdayTimeFormatter(),
    ),
  }));
}
