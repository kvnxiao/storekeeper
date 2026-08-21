/**
 * Parses a backend RFC 3339 timestamp into an instant, returning null for an
 * absent or unparseable value rather than throwing.
 */
export function parseInstant(value: string | null | undefined): Temporal.Instant | null {
  if (!value) {
    return null;
  }
  try {
    return Temporal.Instant.from(value);
  } catch {
    return null;
  }
}
