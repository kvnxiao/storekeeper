//! Retry helpers for daily reward operations.
//!
//! Provides exponential-backoff retry for network calls that may fail
//! due to transient issues (timeouts, DNS, connection resets, etc.).

use std::future::Future;

use storekeeper_client_core::retry::RetryConfig;

/// Retries a fallible async operation with exponential backoff.
///
/// Only retries on transient network errors (see [`is_retryable_error`]).
/// Non-retryable errors (auth failures, rate limits, etc.) propagate immediately.
pub async fn retry_with_backoff<F, Fut>(operation: F) -> anyhow::Result<serde_json::Value>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = anyhow::Result<serde_json::Value>>,
{
    let config = RetryConfig::default();
    storekeeper_client_core::retry::retry_with_backoff(&config, operation, is_retryable_error).await
}

/// Determines if an error is retryable by inspecting the error chain.
///
/// Walks the `anyhow` error chain looking for `reqwest` errors that indicate
/// transient network issues (timeouts, connection errors, etc.).
fn is_retryable_error(error: &anyhow::Error) -> bool {
    // Walk the error chain for typed reqwest errors
    for cause in error.chain() {
        if let Some(reqwest_err) = cause.downcast_ref::<reqwest::Error>() {
            if storekeeper_client_core::is_transient_reqwest_error(reqwest_err) {
                return true;
            }
        }
    }

    // Fallback: pattern-match on the display string for errors that don't
    // preserve typed info through the chain
    let msg = error.to_string().to_lowercase();
    msg.contains("timeout")
        || msg.contains("connection")
        || msg.contains("network")
        || msg.contains("refused")
        || msg.contains("dns")
        || msg.contains("reset")
        || msg.contains("unreachable")
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // is_retryable_error tests
    // =========================================================================

    #[test]
    fn retryable_dns_in_message() {
        let err = anyhow::anyhow!("dns resolution failed");
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn retryable_reset_in_message() {
        let err = anyhow::anyhow!("connection reset by peer");
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn retryable_unreachable_in_message() {
        let err = anyhow::anyhow!("host unreachable");
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn retryable_timeout_in_message() {
        let err = anyhow::anyhow!("request timeout");
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn retryable_connection_in_message() {
        let err = anyhow::anyhow!("connection dropped");
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn retryable_network_in_message() {
        let err = anyhow::anyhow!("temporary network issue");
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn retryable_refused_in_message() {
        let err = anyhow::anyhow!("connection refused by host");
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn not_retryable_auth_error() {
        let err = anyhow::anyhow!("authentication failed: invalid token");
        assert!(!is_retryable_error(&err));
    }

    #[test]
    fn not_retryable_rate_limit() {
        let err = anyhow::anyhow!("rate limit exceeded");
        assert!(!is_retryable_error(&err));
    }

    #[test]
    fn not_retryable_empty() {
        let err = anyhow::anyhow!("");
        assert!(!is_retryable_error(&err));
    }

    // =========================================================================
    // retry_with_backoff tests
    // =========================================================================

    #[tokio::test(start_paused = true)]
    async fn retry_succeeds_on_first_attempt() {
        let result = retry_with_backoff(|| async { Ok(serde_json::json!({"ok": true})) }).await;
        assert!(result.is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn retry_non_retryable_error_fails_immediately() {
        let mut calls = 0u32;
        let result = retry_with_backoff(|| {
            calls += 1;
            async { Err(anyhow::anyhow!("authentication failed")) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls, 1, "non-retryable error should not be retried");
    }

    #[tokio::test(start_paused = true)]
    async fn retry_recovers_from_transient_error() {
        let mut calls = 0u32;
        let result = retry_with_backoff(|| {
            calls += 1;
            async move {
                if calls <= 1 {
                    Err(anyhow::anyhow!("connection timeout"))
                } else {
                    Ok(serde_json::json!({"ok": true}))
                }
            }
        })
        .await;
        assert!(result.is_ok());
        assert_eq!(calls, 2);
    }
}
