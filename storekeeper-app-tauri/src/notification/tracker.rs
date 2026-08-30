//! Notification cooldown tracker for (game, resource) pairs.

use super::resource_extractor::ResourceInfo;
use jiff::SignedDuration;
use jiff::Timestamp;
use std::collections::HashMap;
use storekeeper_core::GameId;
use storekeeper_core::ResourceNotificationConfig;

/// Pre-built `HashMap` key for a (game, resource) pair.
pub type NotifyKey = (GameId, String);

/// Result of a `should_notify` check.
pub enum NotifyAction {
    /// Do not send a notification.
    Skip,
    /// Send a notification; includes the pre-built key for `record`.
    Notify(NotifyKey),
}

#[cfg(test)]
impl NotifyAction {
    fn is_notify(&self) -> bool {
        matches!(self, Self::Notify(_))
    }
}

/// Tracks notification cooldown state per (game, resource) pair.
#[derive(Default)]
pub struct NotificationTracker {
    cooldowns: HashMap<NotifyKey, Timestamp>,
}

impl NotificationTracker {
    /// Decides whether a notification should fire for this resource.
    ///
    /// Returns `Skip` (and clears cooldown) when the resource is outside the
    /// notification window. Returns `Skip` when still within cooldown. Returns
    /// `Notify(key)` when the resource is in-window/full and cooldown has
    /// expired or no prior notification exists.
    ///
    /// When `cooldown_minutes` is 0, only one notification fires per window
    /// entry - no recurring reminders until the resource leaves and re-enters.
    pub fn should_notify(
        &mut self,
        game_id: GameId,
        resource_type: &str,
        config: &ResourceNotificationConfig,
        info: &ResourceInfo,
        now: Timestamp,
    ) -> NotifyAction {
        let in_window = match (config.notify_at_value, config.notify_minutes_before_full) {
            // In value-threshold mode, floor the step count to avoid opening
            // the window before `estimated_current` reaches the threshold; a
            // ceiling opens it one step early.
            (Some(threshold), _) => {
                if let (Some(max), Some(step_seconds)) = (info.max, info.regen_rate_seconds) {
                    let step_units = info.regen_step_units.unwrap_or(1).max(1);
                    let steps_remaining = max.saturating_sub(threshold) / step_units;
                    let effective_minutes = i64::try_from(
                        steps_remaining
                            .checked_mul(step_seconds)
                            .map_or(u64::MAX, |v| v / 60),
                    )
                    .unwrap_or(i64::MAX);
                    // Build the window in seconds with saturating arithmetic so
                    // an extreme `effective_minutes` can't
                    // overflow the duration.
                    let window = SignedDuration::from_secs(effective_minutes.saturating_mul(60));
                    info.is_complete || info.completion_at.duration_since(now) <= window
                } else {
                    info.current.is_some_and(|c| c >= threshold) || info.is_complete
                }
            }
            // Minutes-before-full mode.
            (None, Some(minutes)) => {
                let window = SignedDuration::from_mins(i64::from(minutes));
                info.is_complete || info.completion_at.duration_since(now) <= window
            }
            // Neither set: notify only when the resource is full.
            (None, None) => info.is_complete,
        };

        let key = (game_id, resource_type.to_string());

        if !in_window {
            self.cooldowns.remove(&key);
            return NotifyAction::Skip;
        }

        if let Some(last_notified) = self.cooldowns.get(&key).copied() {
            // cooldown_minutes == 0 means "notify once, don't repeat"
            if config.cooldown_minutes == 0 {
                return NotifyAction::Skip;
            }
            let cooldown = SignedDuration::from_mins(i64::from(config.cooldown_minutes));
            if now.duration_since(last_notified) < cooldown {
                return NotifyAction::Skip;
            }
        }

        NotifyAction::Notify(key)
    }

    /// Records that a notification was sent for the given key.
    pub fn record(&mut self, key: NotifyKey, now: Timestamp) {
        self.cooldowns.insert(key, now);
    }

    /// Clears cooldown entries for a specific game.
    ///
    /// Removes all (game, resource) cooldowns matching the given game ID.
    pub fn clear_for_game(&mut self, game_id: GameId) {
        self.cooldowns.retain(|(id, _), _| *id != game_id);
    }

    /// Clears all cooldown entries.
    #[expect(
        dead_code,
        reason = "kept for API symmetry with clear_for_game; not currently called"
    )]
    pub fn clear_all(&mut self) {
        self.cooldowns.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stub_config(threshold_min: u32, cooldown_min: u32) -> ResourceNotificationConfig {
        ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: if threshold_min > 0 {
                Some(threshold_min)
            } else {
                None
            },
            notify_at_value: None,
            cooldown_minutes: cooldown_min,
        }
    }

    fn stub_info(completion_at: Timestamp, is_complete: bool) -> ResourceInfo {
        ResourceInfo {
            completion_at,
            is_complete,
            current: None,
            max: None,
            regen_rate_seconds: None,
            regen_step_units: None,
        }
    }

    fn key(game_id: GameId, resource_type: &str) -> NotifyKey {
        (game_id, resource_type.to_string())
    }

    #[test]
    fn not_in_window_clears_and_returns_false() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let game = GameId::GenshinImpact;
        let config = stub_config(60, 10);

        tracker.record(key(game, "resin"), now - SignedDuration::from_hours(1));

        let info = stub_info(now + SignedDuration::from_hours(2), false);
        assert!(
            !tracker
                .should_notify(game, "resin", &config, &info, now)
                .is_notify()
        );

        let in_window_info = stub_info(now + SignedDuration::from_mins(30), false);
        assert!(
            tracker
                .should_notify(game, "resin", &config, &in_window_info, now)
                .is_notify()
        );
    }

    #[test]
    fn in_window_first_time_returns_true() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let config = stub_config(60, 10);
        let info = stub_info(now + SignedDuration::from_mins(30), false);

        assert!(
            tracker
                .should_notify(GameId::GenshinImpact, "resin", &config, &info, now)
                .is_notify()
        );
    }

    #[test]
    fn in_window_within_cooldown_returns_false() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let game = GameId::GenshinImpact;
        let config = stub_config(60, 10);

        tracker.record(key(game, "resin"), now);

        let info = stub_info(now + SignedDuration::from_mins(30), false);
        // 5 minutes later, still within 10-minute cooldown
        let later = now + SignedDuration::from_mins(5);
        assert!(
            !tracker
                .should_notify(game, "resin", &config, &info, later)
                .is_notify()
        );
    }

    #[test]
    fn in_window_after_cooldown_expired_returns_true() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let game = GameId::HonkaiStarRail;
        let config = stub_config(60, 10);

        tracker.record(key(game, "trailblaze_power"), now);

        let info = stub_info(now + SignedDuration::from_mins(30), false);
        // 11 minutes later, past 10-minute cooldown
        let later = now + SignedDuration::from_mins(11);
        assert!(
            tracker
                .should_notify(game, "trailblaze_power", &config, &info, later)
                .is_notify()
        );
    }

    #[test]
    fn at_full_returns_true() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let config = stub_config(60, 10);
        let info = stub_info(now - SignedDuration::from_secs(1), true);

        assert!(
            tracker
                .should_notify(GameId::ZenlessZoneZero, "battery", &config, &info, now)
                .is_notify()
        );
    }

    #[test]
    fn clear_resets_state() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let game = GameId::WutheringWaves;
        let config = stub_config(60, 10);

        tracker.record(key(game, "waveplates"), now);

        let info = stub_info(now + SignedDuration::from_mins(30), false);
        assert!(
            !tracker
                .should_notify(game, "waveplates", &config, &info, now)
                .is_notify()
        );

        tracker.cooldowns.remove(&key(game, "waveplates"));
        assert!(
            tracker
                .should_notify(game, "waveplates", &config, &info, now)
                .is_notify()
        );
    }

    #[test]
    fn zero_cooldown_notifies_once_then_stops() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let game = GameId::GenshinImpact;
        let config = stub_config(60, 0); // cooldown_minutes = 0

        let info = stub_info(now + SignedDuration::from_mins(30), false);

        assert!(
            tracker
                .should_notify(game, "resin", &config, &info, now)
                .is_notify()
        );
        tracker.record(key(game, "resin"), now);

        let much_later = now + SignedDuration::from_hours(24);
        assert!(
            !tracker
                .should_notify(game, "resin", &config, &info, much_later)
                .is_notify()
        );
    }

    #[test]
    fn zero_cooldown_resets_on_leaving_window() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let game = GameId::GenshinImpact;
        let config = stub_config(60, 0);

        let in_window = stub_info(now + SignedDuration::from_mins(30), false);
        assert!(
            tracker
                .should_notify(game, "resin", &config, &in_window, now)
                .is_notify()
        );
        tracker.record(key(game, "resin"), now);

        let out_of_window = stub_info(now + SignedDuration::from_hours(5), false);
        assert!(
            !tracker
                .should_notify(game, "resin", &config, &out_of_window, now)
                .is_notify()
        );

        assert!(
            tracker
                .should_notify(game, "resin", &config, &in_window, now)
                .is_notify()
        );
    }

    #[test]
    fn value_threshold_with_regen_rate() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: Some(140),
            cooldown_minutes: 10,
        };

        // Resin: max=160, rate=480s/unit. threshold=140, remaining=20 units,
        // 20*480/60=160 min
        let info = ResourceInfo {
            completion_at: now + SignedDuration::from_mins(100), // within 160 min window
            is_complete: false,
            current: Some(120),
            max: Some(160),
            regen_rate_seconds: Some(480),
            regen_step_units: Some(1),
        };

        assert!(
            tracker
                .should_notify(GameId::GenshinImpact, "resin", &config, &info, now)
                .is_notify()
        );
    }

    #[test]
    fn value_threshold_window_never_fires_below_the_threshold() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: Some(2000),
            cooldown_minutes: 10,
        };
        let realm = |secs_to_full: i64| ResourceInfo {
            completion_at: now + SignedDuration::from_secs(secs_to_full),
            is_complete: false,
            current: Some(1980),
            max: Some(2400),
            regen_rate_seconds: Some(3600),
            regen_step_units: Some(30),
        };

        let early = realm(14 * 3600);
        assert_eq!(early.estimated_current(now), Some(1980));
        assert!(
            !tracker
                .should_notify(
                    GameId::GenshinImpact,
                    "realm_currency",
                    &config,
                    &early,
                    now
                )
                .is_notify()
        );

        let at_threshold = realm(13 * 3600);
        assert_eq!(at_threshold.estimated_current(now), Some(2010));
        assert!(
            tracker
                .should_notify(
                    GameId::GenshinImpact,
                    "realm_currency",
                    &config,
                    &at_threshold,
                    now
                )
                .is_notify()
        );
    }

    #[test]
    fn value_threshold_not_in_window() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: Some(140),
            cooldown_minutes: 10,
        };

        // threshold=140, remaining=20 units, 20*480/60=160 min.
        // time_to_full=200 > 160
        let info = ResourceInfo {
            completion_at: now + SignedDuration::from_mins(200),
            is_complete: false,
            current: Some(100),
            max: Some(160),
            regen_rate_seconds: Some(480),
            regen_step_units: Some(1),
        };

        assert!(
            !tracker
                .should_notify(GameId::GenshinImpact, "resin", &config, &info, now)
                .is_notify()
        );
    }

    #[test]
    fn value_threshold_regen_rate_math_boundary() {
        // threshold=140, max=160, rate=480s/unit: units_remaining = 20 and
        // effective_minutes = 20 * 480 / 60 = 160.
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: Some(140),
            cooldown_minutes: 10,
        };

        let at_boundary = ResourceInfo {
            completion_at: now + SignedDuration::from_mins(160),
            is_complete: false,
            current: Some(120),
            max: Some(160),
            regen_rate_seconds: Some(480),
            regen_step_units: Some(1),
        };
        assert!(
            tracker
                .should_notify(GameId::GenshinImpact, "resin", &config, &at_boundary, now)
                .is_notify()
        );

        tracker
            .cooldowns
            .remove(&key(GameId::GenshinImpact, "resin"));

        let outside_boundary = ResourceInfo {
            completion_at: now + SignedDuration::from_mins(161),
            is_complete: false,
            current: Some(119),
            max: Some(160),
            regen_rate_seconds: Some(480),
            regen_step_units: Some(1),
        };
        assert!(
            !tracker
                .should_notify(
                    GameId::GenshinImpact,
                    "resin",
                    &config,
                    &outside_boundary,
                    now
                )
                .is_notify()
        );
    }

    #[test]
    fn value_threshold_fallback_direct_comparison() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: Some(140),
            cooldown_minutes: 10,
        };

        let info = ResourceInfo {
            completion_at: now + SignedDuration::from_hours(1),
            is_complete: false,
            current: Some(145),
            max: Some(160),
            regen_rate_seconds: None,
            regen_step_units: None,
        };

        assert!(
            tracker
                .should_notify(GameId::GenshinImpact, "resin", &config, &info, now)
                .is_notify()
        );
    }

    #[test]
    fn value_threshold_fallback_below_threshold() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: Some(140),
            cooldown_minutes: 10,
        };

        let info = ResourceInfo {
            completion_at: now + SignedDuration::from_hours(1),
            is_complete: false,
            current: Some(100),
            max: Some(160),
            regen_rate_seconds: None,
            regen_step_units: None,
        };

        assert!(
            !tracker
                .should_notify(GameId::GenshinImpact, "resin", &config, &info, now)
                .is_notify()
        );
    }

    #[test]
    fn neither_threshold_only_notifies_when_full() {
        let mut tracker = NotificationTracker::default();
        let now = Timestamp::now();
        let config = ResourceNotificationConfig {
            enabled: true,
            notify_minutes_before_full: None,
            notify_at_value: None,
            cooldown_minutes: 10,
        };

        let info = stub_info(now + SignedDuration::from_mins(5), false);
        assert!(
            !tracker
                .should_notify(GameId::GenshinImpact, "resin", &config, &info, now)
                .is_notify()
        );

        let full_info = stub_info(now - SignedDuration::from_secs(1), true);
        assert!(
            tracker
                .should_notify(GameId::GenshinImpact, "resin", &config, &full_info, now)
                .is_notify()
        );
    }
}
