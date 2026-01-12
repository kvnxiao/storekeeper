import { atom } from "jotai";
import { atomEffect } from "jotai-effect";

// =============================================================================
// Tick - Updates every minute for real-time countdown display
// =============================================================================

/** Base tick atom - stores unix timestamp in milliseconds */
const base = atom<number>(Date.now());

/** Effect atom that sets up the interval */
const tickEffect = atomEffect((_get, set) => {
  const interval = setInterval(() => {
    set(base, Date.now());
  }, 60_000); // 1 minute

  return () => clearInterval(interval);
});

/** Current tick timestamp - triggers effect when read */
export const currentTick = atom((get) => {
  get(tickEffect);
  return get(base);
});
