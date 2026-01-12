// Augment global Intl namespace with DurationFormat from @formatjs/intl-durationformat
import type {
  DurationFormat as DurationFormatImpl,
  DurationFormatOptions,
} from "@formatjs/intl-durationformat";

declare global {
  namespace Intl {
    type DurationFormat = DurationFormatImpl;
    const DurationFormat: {
      new (
        locales?: string | string[],
        options?: DurationFormatOptions,
      ): DurationFormat;
    };
  }
}
