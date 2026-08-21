//! Retry utilities with exponential backoff and jitter.

use rand::RngExt;
use std::future::Future;
use std::time::Duration;

/// Default maximum number of retry attempts.
pub const DEFAULT_MAX_RETRIES: u32 = 3;
/// Default base delay in milliseconds for exponential backoff.
pub const DEFAULT_BASE_DELAY_MS: u64 = 500;
/// Default maximum delay in milliseconds (cap for exponential growth).
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
    #[must_use]
    pub fn new(max_retries: u32, base_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_retries,
            base_delay_ms,
            max_delay_ms,
        }
    }

    /// Calculates the delay for a given attempt with exponential backoff and
    /// jitter.
    ///
    /// Formula: `min(base_delay * 2^attempt + jitter, max_delay)`
    /// where jitter is a random value between 0 and `base_delay`.
    #[must_use]
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base = self
            .base_delay_ms
            .saturating_mul(2u64.saturating_pow(attempt));
        let capped = base.min(self.max_delay_ms);

        let jitter = rand::rng().random_range(0..=self.base_delay_ms);
        let with_jitter = capped.saturating_add(jitter).min(self.max_delay_ms);

        Duration::from_millis(with_jitter)
    }

    /// Returns whether another retry should be attempted.
    #[must_use]
    pub fn should_retry(&self, current_attempt: u32) -> bool {
        current_attempt < self.max_retries
    }
}

/// Retries an async operation with exponential backoff.
///
/// Executes `operation` and retries on failures where `is_retryable` returns
/// `true`, up to `config.max_retries` times with exponential backoff delays
/// between attempts.
///
/// Total attempts = 1 initial + `config.max_retries` retries.
///
/// # Errors
///
/// Returns the last error if all retries are exhausted, or the first
/// non-retryable error.
pub async fn retry_with_backoff<T, E, F, Fut>(
    config: &RetryConfig,
    mut operation: F,
    is_retryable: impl Fn(&E) -> bool,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if is_retryable(&e) && config.should_retry(attempt) => {
                let delay = config.delay_for_attempt(attempt);
                tracing::warn!(
                    attempt = attempt,
                    max_retries = config.max_retries,
                    delay_ms = delay.as_millis(),
                    error = %e,
                    "Retrying after transient error"
                );
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_the_declared_constants() {
        let config = RetryConfig::default();

        assert_eq!(config.max_retries, DEFAULT_MAX_RETRIES);
        assert_eq!(config.base_delay_ms, DEFAULT_BASE_DELAY_MS);
        assert_eq!(config.max_delay_ms, DEFAULT_MAX_DELAY_MS);
    }

    #[test]
    fn new_stores_its_arguments() {
        for (retries, base, max) in [(5, 1000, 60_000), (0, 0, 0)] {
            let config = RetryConfig::new(retries, base, max);

            assert_eq!(config.max_retries, retries);
            assert_eq!(config.base_delay_ms, base);
            assert_eq!(config.max_delay_ms, max);
        }
    }

    #[test]
    fn should_retry_until_the_attempt_reaches_max_retries() {
        let config = RetryConfig::new(3, 500, 30_000);

        for (attempt, expected) in [
            (0, true),
            (1, true),
            (2, true),
            (3, false),
            (4, false),
            (100, false),
        ] {
            assert_eq!(
                config.should_retry(attempt),
                expected,
                "attempt {attempt} against max_retries 3"
            );
        }
    }

    #[test]
    fn should_not_retry_when_max_retries_is_zero() {
        let config = RetryConfig::new(0, 500, 30_000);

        assert!(!config.should_retry(0));
    }

    #[test]
    fn delay_grows_exponentially_below_the_cap() {
        let config = RetryConfig::new(5, 100, 1_000_000);

        for attempt in 0..5 {
            let floor = 100 * 2u128.pow(attempt);
            let delay = config.delay_for_attempt(attempt).as_millis();

            assert!(
                (floor..=floor + 100).contains(&delay),
                "attempt {attempt} should land in [{floor}, {}], got {delay}",
                floor + 100
            );
        }
    }

    #[test]
    fn delay_never_exceeds_max_delay() {
        let config = RetryConfig::new(10, 1000, 5000);

        for attempt in 0..10 {
            assert!(config.delay_for_attempt(attempt).as_millis() <= 5000);
        }
    }

    #[test]
    fn delay_varies_across_calls_within_the_jitter_range() {
        let config = RetryConfig::new(3, 500, 30_000);

        let delays: Vec<u128> = (0..20)
            .map(|_| config.delay_for_attempt(0).as_millis())
            .collect();

        for &delay in &delays {
            assert!(
                (500..=1000).contains(&delay),
                "attempt 0 should land in [500, 1000], got {delay}"
            );
        }

        let first = *delays.first().expect("twenty delays were recorded");
        assert!(
            delays.iter().any(|&d| d != first),
            "twenty draws from a 501-value jitter range should not all match"
        );
    }

    #[test]
    fn delay_saturates_instead_of_overflowing() {
        let config = RetryConfig::new(100, 500, 30_000);

        for attempt in [50, 99] {
            assert!(config.delay_for_attempt(attempt).as_millis() <= 30_000);
        }
    }

    #[test]
    fn delay_with_zero_base_stays_under_max() {
        let config = RetryConfig::new(3, 0, 1000);

        assert!(config.delay_for_attempt(0).as_millis() <= 1000);
    }

    #[test]
    fn delay_with_zero_max_is_zero() {
        let config = RetryConfig::new(3, 500, 0);

        assert_eq!(config.delay_for_attempt(0).as_millis(), 0);
    }

    #[tokio::test(start_paused = true)]
    async fn retry_with_backoff_succeeds_immediately() {
        let config = RetryConfig::new(3, 100, 1000);
        let result =
            retry_with_backoff(&config, || async { Ok::<_, String>(42) }, |_: &String| true).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test(start_paused = true)]
    async fn retry_with_backoff_retries_and_recovers() {
        let config = RetryConfig::new(3, 100, 1000);
        let mut calls = 0u32;
        let result = retry_with_backoff(
            &config,
            || {
                calls += 1;
                let c = calls;
                async move {
                    if c <= 2 {
                        Err("transient".to_string())
                    } else {
                        Ok(99)
                    }
                }
            },
            |_: &String| true,
        )
        .await;
        assert_eq!(result, Ok(99));
        assert_eq!(calls, 3);
    }

    #[tokio::test(start_paused = true)]
    async fn retry_with_backoff_non_retryable_fails_immediately() {
        let config = RetryConfig::new(3, 100, 1000);
        let mut calls = 0u32;
        let result = retry_with_backoff(
            &config,
            || {
                calls += 1;
                async { Err::<i32, _>("fatal".to_string()) }
            },
            |e: &String| e != "fatal",
        )
        .await;
        assert_eq!(result, Err("fatal".to_string()));
        assert_eq!(calls, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn retry_with_backoff_exhausts_retries() {
        let config = RetryConfig::new(2, 100, 1000);
        let mut calls = 0u32;
        let result = retry_with_backoff(
            &config,
            || {
                calls += 1;
                async { Err::<i32, _>("always fails".to_string()) }
            },
            |_: &String| true,
        )
        .await;
        assert_eq!(result, Err("always fails".to_string()));
        assert_eq!(calls, 3, "one initial attempt plus two retries");
    }
}
