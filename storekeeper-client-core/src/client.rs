//! HTTP client builder utilities.

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::RetryTransientMiddleware;
use reqwest_retry::policies::ExponentialBackoff;

use crate::error::{ClientError, Result};
use crate::retry::{DEFAULT_BASE_DELAY_MS, DEFAULT_MAX_DELAY_MS};

/// Default User-Agent string for API clients.
pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Builder for creating configured HTTP clients.
///
/// This builder provides a fluent API for configuring common HTTP client
/// options like headers, timeouts, and other settings.
///
/// # Example
///
/// ```rust,ignore
/// use storekeeper_client_core::HttpClientBuilder;
///
/// let client = HttpClientBuilder::new()
///     .header_static("x-custom-header", "value")
///     .build()?;
/// ```
#[derive(Debug, Default)]
pub struct HttpClientBuilder {
    headers: HeaderMap,
    user_agent: Option<String>,
}

impl HttpClientBuilder {
    /// Creates a new HTTP client builder with default settings.
    #[must_use = "builder must be used to create an HTTP client"]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the User-Agent header.
    ///
    /// If not called, uses the default User-Agent string.
    #[must_use = "this returns the modified builder"]
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Adds a header to the default headers.
    ///
    /// # Errors
    ///
    /// Returns an error if the header value is invalid.
    pub fn header(
        mut self,
        name: impl reqwest::header::IntoHeaderName,
        value: &str,
    ) -> Result<Self> {
        let header_value = HeaderValue::from_str(value)
            .map_err(|e| ClientError::invalid_config(format!("Invalid header value: {e}")))?;
        self.headers.insert(name, header_value);
        Ok(self)
    }

    /// Adds a static header to the default headers.
    ///
    /// Use this when the header value is a compile-time constant.
    #[must_use = "this returns the modified builder"]
    pub fn header_static(
        mut self,
        name: impl reqwest::header::IntoHeaderName,
        value: &'static str,
    ) -> Self {
        self.headers.insert(name, HeaderValue::from_static(value));
        self
    }

    /// Builds the configured `reqwest::Client`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn build(mut self) -> Result<reqwest::Client> {
        // Set User-Agent
        let user_agent = self.user_agent.as_deref().unwrap_or(DEFAULT_USER_AGENT);
        let user_agent_value = HeaderValue::from_str(user_agent)
            .map_err(|e| ClientError::invalid_config(format!("Invalid User-Agent: {e}")))?;
        self.headers.insert(USER_AGENT, user_agent_value);

        reqwest::Client::builder()
            .default_headers(self.headers)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(ClientError::HttpRequest)
    }

    /// Builds a client with retry middleware for HTTP-level failures.
    ///
    /// This adds automatic retry with exponential backoff for transient errors:
    /// - HTTP 5xx server errors
    /// - Connection timeouts
    /// - Network failures
    ///
    /// Note: This does NOT retry on 4xx client errors or successful responses.
    /// For application-level retries (e.g., based on response body content),
    /// use the [`RetryConfig`](crate::retry::RetryConfig) utility directly.
    ///
    /// # Arguments
    ///
    /// * `max_retries` - Maximum number of retry attempts
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn build_with_retry(self, max_retries: u32) -> Result<ClientWithMiddleware> {
        let base_client = self.build()?;

        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(
                Duration::from_millis(DEFAULT_BASE_DELAY_MS),
                Duration::from_millis(DEFAULT_MAX_DELAY_MS),
            )
            .jitter(reqwest_retry::Jitter::Bounded)
            .build_with_max_retries(max_retries);

        let client = ClientBuilder::new(base_client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(client)
    }
}
