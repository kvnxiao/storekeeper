/**
 * Sets the direction for the next route transition, read by the
 * `[data-view-transition-direction]` rules in `styles.css`.
 */
export function setViewTransitionDirection(direction: "forward" | "back"): void {
  document.documentElement.dataset.viewTransitionDirection = direction;
}
