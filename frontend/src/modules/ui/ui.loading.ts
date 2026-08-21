import { type Accessor, createEffect, createSignal, on, onCleanup } from "solid-js";

/**
 * Shortest time a loading indicator stays at full strength, long enough for
 * one full `mask-shimmer` sweep in `styles.css`. A provider that answers in
 * milliseconds would otherwise flash the indicator on and off.
 */
export const LOADING_MIN_MS = 1_200;

/**
 * How long the indicator takes to leave, matching the `--shimmer-dim`
 * transition in `styles.css`. Dropping the mask any sooner returns the card to
 * full opacity mid-sweep.
 */
export const LOADING_FADE_MS = 250;

export type LoadingPhase = "active" | "fading";

/**
 * Tracks `isLoading` as a phase that holds for `LOADING_MIN_MS` and then fades.
 *
 * Loading that resumes during either the hold or the fade returns the phase to
 * `active` and restarts the hold. Call this under a reactive root; the pending
 * timers are cleared when that root disposes.
 */
export function createLoadingPhase(
  isLoading: Accessor<boolean>,
): Accessor<LoadingPhase | undefined> {
  const [phase, setPhase] = createSignal<LoadingPhase | undefined>();

  let holdTimeout: ReturnType<typeof setTimeout> | undefined;
  let fadeTimeout: ReturnType<typeof setTimeout> | undefined;
  let held = false;

  function startFade(): void {
    setPhase("fading");
    fadeTimeout = setTimeout(() => setPhase(undefined), LOADING_FADE_MS);
  }

  createEffect(
    on(isLoading, (loading) => {
      if (loading) {
        clearTimeout(fadeTimeout);
        if (phase() !== "active") {
          held = true;
          clearTimeout(holdTimeout);
          holdTimeout = setTimeout(() => {
            held = false;
            if (!isLoading()) {
              startFade();
            }
          }, LOADING_MIN_MS);
        }
        setPhase("active");
        return;
      }
      if (phase() !== "active" || held) {
        return;
      }
      startFade();
    }),
  );

  onCleanup(() => {
    clearTimeout(holdTimeout);
    clearTimeout(fadeTimeout);
  });

  return phase;
}
