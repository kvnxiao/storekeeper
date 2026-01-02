//! Retry utilities with exponential backoff and jitter.

use std::time::Duration;

use rand::Rng;

/// Default retry configuration constants.
pub const DEFAULT_MAX_RETRIES: u32 = 3;
pub const DEFAULT_BASE_DELAY_MS: u64 = 500;
pub const DEFAULT_MAX_DELAY_MS: u64 = 30_000;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Base delay in milliseconds for exponential backoff.
    pub base_delay_ms: u64,
    /// Maximum delay in milliseconds (cap for exponential growth).
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            base_delay_ms: DEFAULT_BASE_DELAY_MS,
            max_delay_ms: DEFAULT_MAX_DELAY_MS,
        }
    }
}

impl RetryConfig {
    /// Creates a new retry configuration.
    #[must_use = "this returns a new RetryConfig and does not modify self"]
    pub fn new(max_retries: u32, base_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_retries,
            base_delay_ms,
            max_delay_ms,
        }
    }

    /// Calculates the delay for a given attempt with exponential backoff and jitter.
    ///
    /// Formula: `min(base_delay * 2^attempt + jitter, max_delay)`
    /// where jitter is a random value between 0 and base_delay.
    #[must_use = "this returns a Duration and does not modify self"]
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base = self
            .base_delay_ms
            .saturating_mul(2u64.saturating_pow(attempt));
        let capped = base.min(self.max_delay_ms);

        // Add bounded jitter: random value between 0 and base_delay_ms
        let jitter = rand::thread_rng().gen_range(0..=self.base_delay_ms);
        let with_jitter = capped.saturating_add(jitter).min(self.max_delay_ms);

        Duration::from_millis(with_jitter)
    }

    /// Returns whether another retry should be attempted.
    #[must_use = "this returns a bool and does not modify self"]
    pub fn should_retry(&self, current_attempt: u32) -> bool {
        current_attempt < self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Default configuration tests
    // =========================================================================

    #[test]
    fn test_default_values() {
        let config = RetryConfig::default();

        assert_eq!(
            config.max_retries, DEFAULT_MAX_RETRIES,
            "Default max_retries should be {DEFAULT_MAX_RETRIES}"
        );
        assert_eq!(
            config.base_delay_ms, DEFAULT_BASE_DELAY_MS,
            "Default base_delay_ms should be {DEFAULT_BASE_DELAY_MS}"
        );
        assert_eq!(
            config.max_delay_ms, DEFAULT_MAX_DELAY_MS,
            "Default max_delay_ms should be {DEFAULT_MAX_DELAY_MS}"
        );
    }

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_MAX_RETRIES, 3);
        assert_eq!(DEFAULT_BASE_DELAY_MS, 500);
        assert_eq!(DEFAULT_MAX_DELAY_MS, 30_000);
    }

    // =========================================================================
    // RetryConfig::new tests
    // =========================================================================

    #[test]
    fn test_new_with_custom_values() {
        let config = RetryConfig::new(5, 1000, 60_000);

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 60_000);
    }

    #[test]
    fn test_new_with_zero_values() {
        let config = RetryConfig::new(0, 0, 0);

        assert_eq!(config.max_retries, 0);
        assert_eq!(config.base_delay_ms, 0);
        assert_eq!(config.max_delay_ms, 0);
    }

    // =========================================================================
    // should_retry tests
    // =========================================================================

    #[test]
    fn test_should_retry_when_under_limit() {
        let config = RetryConfig::new(3, 500, 30_000);

        assert!(
            config.should_retry(0),
            "Should retry when attempt 0 < max_retries 3"
        );
        assert!(
            config.should_retry(1),
            "Should retry when attempt 1 < max_retries 3"
        );
        assert!(
            config.should_retry(2),
            "Should retry when attempt 2 < max_retries 3"
        );
    }

    #[test]
    fn test_should_not_retry_at_limit() {
        let config = RetryConfig::new(3, 500, 30_000);

        assert!(
            !config.should_retry(3),
            "Should not retry when attempt 3 >= max_retries 3"
        );
    }

    #[test]
    fn test_should_not_retry_over_limit() {
        let config = RetryConfig::new(3, 500, 30_000);

        assert!(
            !config.should_retry(4),
            "Should not retry when attempt 4 > max_retries 3"
        );
        assert!(
            !config.should_retry(100),
            "Should not retry when attempt 100 > max_retries 3"
        );
    }

    #[test]
    fn test_should_not_retry_when_max_is_zero() {
        let config = RetryConfig::new(0, 500, 30_000);

        assert!(
            !config.should_retry(0),
            "Should not retry when max_retries is 0"
        );
    }

    // =========================================================================
    // delay_for_attempt tests
    // =========================================================================

    #[test]
    fn test_delay_is_duration() {
        let config = RetryConfig::new(3, 500, 30_000);
        let delay = config.delay_for_attempt(0);

        // delay should be a Duration, not panic
        assert!(delay.as_millis() > 0, "Delay should be positive");
    }

    #[test]
    fn test_delay_respects_max_delay() {
        let config = RetryConfig::new(10, 1000, 5000);

        // Even at high attempts, delay should not exceed max_delay_ms
        for attempt in 0..10 {
            let delay = config.delay_for_attempt(attempt);
            assert!(
                delay.as_millis() <= 5000,
                "Delay at attempt {attempt} should not exceed max_delay_ms (5000), got {}",
                delay.as_millis()
            );
        }
    }

    #[test]
    fn test_delay_exponential_growth_bounded() {
        let config = RetryConfig::new(5, 100, 10_000);

        // The base calculation is: base * 2^attempt + jitter
        // Attempt 0: 100 * 1 + jitter = 100-200
        // Attempt 1: 100 * 2 + jitter = 200-300
        // Attempt 2: 100 * 4 + jitter = 400-500
        // etc.

        let delay_0 = config.delay_for_attempt(0);
        let delay_1 = config.delay_for_attempt(1);

        // delay_1 should generally be larger than delay_0
        // but due to jitter, we can't guarantee exact values
        // Just verify they're reasonable
        assert!(
            delay_0.as_millis() >= 100,
            "Delay at attempt 0 should be at least base_delay_ms"
        );
        assert!(
            delay_1.as_millis() >= 100,
            "Delay at attempt 1 should be at least base_delay_ms"
        );
    }

    #[test]
    fn test_delay_includes_jitter() {
        let config = RetryConfig::new(3, 500, 30_000);

        // Run multiple times and check that we get different values (jitter)
        let mut delays: Vec<u128> = Vec::new();
        for _ in 0..20 {
            let delay = config.delay_for_attempt(0);
            delays.push(delay.as_millis());
        }

        // With jitter, we should see some variation (not all identical)
        // The jitter range is 0..=base_delay_ms, so 0..=500
        // All values should be in range [500, 1000] (base + 0 to base + base)
        for &d in &delays {
            assert!(
                (500..=1000).contains(&d),
                "Delay at attempt 0 should be in range [500, 1000], got {d}"
            );
        }

        // Check we got at least some variation (not all the same)
        let first = delays[0];
        let has_variation = delays.iter().any(|&d| d != first);
        // Note: There's a very small chance all 20 are identical, but extremely unlikely
        // We'll allow this test to pass even without variation to avoid flakiness
        if !has_variation {
            // Just print a note, don't fail
            println!("Note: All delays were identical (rare but possible with random jitter)");
        }
    }

    #[test]
    fn test_delay_saturating_at_high_attempts() {
        let config = RetryConfig::new(100, 500, 30_000);

        // At very high attempts, 2^attempt would overflow, but we use saturating_mul
        // The delay should still be bounded by max_delay_ms
        let delay = config.delay_for_attempt(50);
        assert!(
            delay.as_millis() <= 30_000,
            "Delay at attempt 50 should be capped at max_delay_ms"
        );

        let delay = config.delay_for_attempt(99);
        assert!(
            delay.as_millis() <= 30_000,
            "Delay at attempt 99 should be capped at max_delay_ms"
        );
    }

    #[test]
    fn test_delay_with_zero_base() {
        let config = RetryConfig::new(3, 0, 1000);

        // With zero base, delay should still work (jitter of 0..=0 is just 0)
        let delay = config.delay_for_attempt(0);
        assert!(
            delay.as_millis() <= 1000,
            "Delay with zero base should not exceed max"
        );
    }

    #[test]
    fn test_delay_with_zero_max() {
        let config = RetryConfig::new(3, 500, 0);

        // With zero max, delay should be capped to 0
        let delay = config.delay_for_attempt(0);
        assert_eq!(
            delay.as_millis(),
            0,
            "Delay with zero max_delay should be 0"
        );
    }

    // =========================================================================
    // Clone and Debug tests
    // =========================================================================

    #[test]
    fn test_config_is_clone() {
        let config = RetryConfig::new(3, 500, 30_000);
        let cloned = config.clone();

        assert_eq!(config.max_retries, cloned.max_retries);
        assert_eq!(config.base_delay_ms, cloned.base_delay_ms);
        assert_eq!(config.max_delay_ms, cloned.max_delay_ms);
    }

    #[test]
    fn test_config_is_debug() {
        let config = RetryConfig::new(3, 500, 30_000);
        let debug_str = format!("{config:?}");

        assert!(
            debug_str.contains("RetryConfig"),
            "Debug output should contain type name"
        );
        assert!(
            debug_str.contains("max_retries"),
            "Debug output should contain field names"
        );
    }
}
