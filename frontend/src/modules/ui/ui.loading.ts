import { type Accessor, createEffect, createSignal, on, onCleanup } from "solid-js";

/** Keep the loading phase active for one `mask-shimmer` sweep. */
export const LOADING_MIN_MS = 1_200;

/** Match the `--shimmer-dim` transition duration in `styles.css`. */
export const LOADING_FADE_MS = 250;

export type LoadingPhase = "active" | "fading";

/**
 * Track loading through an active phase and a fade phase.
 *
 * When loading ends before `LOADING_MIN_MS`, keep the phase active until the
 * original hold expires. When loading resumes during the fade, return to
 * `active` and start a new hold. Use this under a reactive owner; disposing the
 * owner clears pending timers.
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
