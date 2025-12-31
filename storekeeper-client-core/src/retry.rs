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
