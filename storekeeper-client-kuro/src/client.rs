//! Kuro Games HTTP client implementation.

use std::time::Instant;

use reqwest::Method;
use reqwest::header::{ACCEPT, CONTENT_TYPE, ORIGIN};
use serde::Serialize;
use serde::de::DeserializeOwned;
use storekeeper_client_core::retry::{DEFAULT_MAX_DELAY_MS, DEFAULT_MAX_RETRIES};
use storekeeper_client_core::{
    ApiResponse, ClientError, ClientWithMiddleware, HttpClientBuilder, KuroApiResponse, RetryConfig,
};

use crate::error::{Error, Result};

/// Base URL for the Kuro Games API.
const KURO_API_BASE: &str = "https://pc-launcher-sdk-api.kurogame.net";

/// Request body for Kuro API calls.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryRoleRequest<'a> {
    game_code: &'a str,
    account_id: &'a str,
    oauth_code: &'a str,
    player_id: &'a str,
    region: &'a str,
}

/// Kuro Games API client.
#[derive(Debug, Clone)]
pub struct KuroClient {
    client: ClientWithMiddleware,
    oauth_code: String,
}

const KURO_API_DEFAULT_BASE_DELAY_MS: u64 = 1500;
const PREFLIGHT_ACCESS_CONTROL_METHOD: &str = "POST";
const PREFLIGHT_ACCESS_CONTROL_HEADERS: &str = "content-type";

impl KuroClient {
    /// Creates a new Kuro Games client with the given OAuth code.
    ///
    /// The client is configured with automatic retry for HTTP-level failures
    /// (5xx errors, timeouts, network errors) using exponential backoff with jitter.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(oauth_code: impl Into<String>) -> Result<Self> {
        let client = HttpClientBuilder::new()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .header_static(ACCEPT, "application/json, text/plain, */*")
            .header_static(CONTENT_TYPE, "application/json")
            .header_static(ORIGIN, "null")
            .build_with_retry(DEFAULT_MAX_RETRIES)
            .map_err(Error::Client)?;

        Ok(Self {
            client,
            oauth_code: oauth_code.into(),
        })
    }

    /// Sends a CORS preflight OPTIONS request to the Kuro API.
    ///
    /// This is required before making the actual POST request due to CORS restrictions.
    ///
    /// # Errors
    ///
    /// Returns an error if the preflight request fails.
    async fn send_preflight(&self, url: &str) -> Result<()> {
        tracing::debug!(url = %url, "Sending CORS preflight OPTIONS request");
        let started = Instant::now();

        let response = self
            .client
            .request(Method::OPTIONS, url)
            .header(
                "Access-Control-Request-Method",
                PREFLIGHT_ACCESS_CONTROL_METHOD,
            )
            .header(
                "Access-Control-Request-Headers",
                PREFLIGHT_ACCESS_CONTROL_HEADERS,
            )
            .send()
            .await?;

        let status = response.status();
        tracing::debug!(
            status = %status,
            elapsed_ms = started.elapsed().as_millis(),
            "Preflight response received"
        );

        if !status.is_success() {
            tracing::warn!(status = %status, "Preflight request failed");
            return Err(Error::Client(ClientError::api_error(
                i32::from(status.as_u16()),
                format!("Preflight request failed with status: {status}"),
            )));
        }

        Ok(())
    }

    /// Performs a single query role attempt without retries.
    async fn query_role_once<T: DeserializeOwned>(
        &self,
        player_id: &str,
        region: &str,
    ) -> Result<T> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let url = format!("{KURO_API_BASE}/game/queryRole?_t={timestamp}");

        // Send CORS preflight request first
        self.send_preflight(&url).await?;

        let body = QueryRoleRequest {
            game_code: "2", // Wuthering Waves
            account_id: "",
            oauth_code: &self.oauth_code,
            player_id,
            region,
        };

        tracing::debug!(
            player_id = %player_id,
            region = %region,
            "Kuro API POST request to queryRole"
        );
        let post_started = Instant::now();

        // Make the POST request
        let response = self.client.post(&url).json(&body).send().await?;

        let status = response.status();
        tracing::debug!(
            status = %status,
            elapsed_ms = post_started.elapsed().as_millis(),
            "Kuro API response received"
        );

        let api_response: KuroApiResponse<serde_json::Value> = response.json().await?;

        // Check response code - Kuro uses code 1005 for retry requests
        match api_response.code {
            code if api_response.is_success() => {
                tracing::debug!(code = code, "Kuro API request successful");
            }
            1005 => {
                tracing::warn!("Kuro API requested retry (code 1005)");
                return Err(Error::RetryRequested);
            }
            _ => {
                tracing::warn!(
                    code = api_response.code,
                    message = %api_response.message,
                    "Kuro API error response"
                );
                return Err(Error::Client(ClientError::api_error(
                    api_response.code,
                    &api_response.message,
                )));
            }
        }

        // The data field contains a map with region as key and JSON string as value
        let data = api_response
            .into_data()
            .ok_or_else(|| Error::Client(ClientError::api_error(0, "Response data is null")))?;

        // Extract the nested JSON string for the region
        let region_data = data
            .get(region)
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::NestedDataParseFailed(format!("No data for region: {region}")))?;

        // Parse the nested JSON string
        serde_json::from_str(region_data)
            .map_err(|e| Error::NestedDataParseFailed(format!("Failed to parse region data: {e}")))
    }

    /// Queries role/character data from the Kuro API.
    ///
    /// This method first sends a CORS preflight OPTIONS request, then makes the actual
    /// POST request to fetch the role data. If the server returns code 1005 (retry requested),
    /// the request will be retried up to 3 times with exponential backoff and jitter.
    ///
    /// HTTP-level failures (5xx, timeouts) are automatically retried by the middleware.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn query_role<T: DeserializeOwned>(
        &self,
        player_id: &str,
        region: &str,
    ) -> Result<T> {
        let retry_config = RetryConfig::new(
            DEFAULT_MAX_RETRIES,
            KURO_API_DEFAULT_BASE_DELAY_MS,
            DEFAULT_MAX_DELAY_MS,
        );
        let mut last_error = None;

        for attempt in 0..=retry_config.max_retries {
            if attempt > 0 {
                // Exponential backoff with jitter
                let delay = retry_config.delay_for_attempt(attempt - 1);
                tracing::info!(
                    attempt = attempt,
                    max_retries = retry_config.max_retries,
                    delay_ms = delay.as_millis(),
                    "Retrying Kuro API request (code 1005)"
                );
                tokio::time::sleep(delay).await;
            }

            match self.query_role_once(player_id, region).await {
                Ok(result) => {
                    if attempt > 0 {
                        tracing::info!(attempt = attempt, "Kuro API request succeeded after retry");
                    }
                    return Ok(result);
                }
                Err(Error::RetryRequested) => {
                    last_error = Some(Error::RetryRequested);
                }
                Err(e) => return Err(e),
            }
        }

        // All retries exhausted
        tracing::error!(
            retries = retry_config.max_retries,
            "Kuro API request failed after all retries (code 1005)"
        );
        Err(last_error.unwrap_or(Error::RetryRequested))
    }

    /// Checks if the client credentials are valid.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication check fails.
    pub async fn check_auth(&self, player_id: &str, region: &str) -> Result<bool> {
        match self
            .query_role::<serde_json::Value>(player_id, region)
            .await
        {
            Ok(_) => Ok(true),
            Err(Error::Client(ClientError::ApiError { .. })) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
