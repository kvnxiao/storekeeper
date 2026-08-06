/**
 * Backend event names, mirroring `AppEvent::as_str` in
 * `storekeeper-app-tauri/src/events.rs`. A test in that file reads this one, so
 * renaming an event on either side fails the Rust test suite instead of
 * silently dropping event delivery.
 */
export const AppEvent = {
  ResourcesUpdated: "resources-updated",
  RefreshStarted: "refresh-started",
  GameResourceUpdated: "game-resource-updated",
  DailyRewardClaimed: "daily-reward-claimed",
  DailyRewardStatusUpdated: "daily-reward-status-updated",
} as const;
