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
    base_url: String,
}

#[cfg(not(test))]
const KURO_API_DEFAULT_BASE_DELAY_MS: u64 = 1500;
#[cfg(test)]
const KURO_API_DEFAULT_BASE_DELAY_MS: u64 = 1;
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
        Self::with_base_url(oauth_code, KURO_API_BASE)
    }

    /// Creates a new Kuro Games client with a custom API base URL.
    ///
    /// This is primarily useful for testing.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn with_base_url(
        oauth_code: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Result<Self> {
        let client = HttpClientBuilder::new()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .header_static(ACCEPT, "application/json, text/plain, */*")
            .header_static(CONTENT_TYPE, "application/json")
            .header_static(ORIGIN, "null")
            .build_with_retry(DEFAULT_MAX_RETRIES)
            .map_err(Error::Client)?;

        let base_url = base_url.into();
        let normalized_base_url = base_url.trim_end_matches('/').to_string();

        Ok(Self {
            client,
            oauth_code: oauth_code.into(),
            base_url: normalized_base_url,
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
        let url = format!("{}/game/queryRole?_t={timestamp}", self.base_url);

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::{Mutex, oneshot};

    use super::*;

    #[derive(Debug, Clone)]
    struct TestRequest {
        method: String,
        target: String,
        headers: HashMap<String, String>,
    }

    #[derive(Debug, Clone)]
    struct TestResponse {
        status: u16,
        body: String,
    }

    struct TestServer {
        base_url: String,
        requests: Arc<Mutex<Vec<TestRequest>>>,
        shutdown_tx: Option<oneshot::Sender<()>>,
    }

    impl TestServer {
        async fn spawn(handler: Arc<dyn Fn(&TestRequest) -> TestResponse + Send + Sync>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind test server");
            let addr = listener.local_addr().expect("get local addr");
            let requests = Arc::new(Mutex::new(Vec::new()));
            let requests_clone = Arc::clone(&requests);
            let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = &mut shutdown_rx => {
                            break;
                        }
                        accepted = listener.accept() => {
                            let Ok((mut stream, _)) = accepted else {
                                break;
                            };
                            let requests = Arc::clone(&requests_clone);
                            let handler = Arc::clone(&handler);
                            tokio::spawn(async move {
                                if let Some(request) = read_request(&mut stream).await {
                                    requests.lock().await.push(request.clone());
                                    let response = handler(&request);
                                    let _ = write_response(&mut stream, response).await;
                                }
                            });
                        }
                    }
                }
            });

            Self {
                base_url: format!("http://{addr}"),
                requests,
                shutdown_tx: Some(shutdown_tx),
            }
        }

        async fn requests(&self) -> Vec<TestRequest> {
            self.requests.lock().await.clone()
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            if let Some(tx) = self.shutdown_tx.take() {
                let _ = tx.send(());
            }
        }
    }

    async fn read_request(stream: &mut TcpStream) -> Option<TestRequest> {
        let mut raw = Vec::new();
        let mut buf = [0_u8; 1024];

        let header_end = loop {
            let read = stream.read(&mut buf).await.ok()?;
            if read == 0 {
                return None;
            }
            raw.extend_from_slice(&buf[..read]);
            if let Some(pos) = find_header_end(&raw) {
                break pos;
            }
        };

        let head = String::from_utf8_lossy(&raw[..header_end]).to_string();
        let mut lines = head.split("\r\n");
        let request_line = lines.next()?.to_string();
        let mut request_line_parts = request_line.split_whitespace();
        let method = request_line_parts.next()?.to_string();
        let target = request_line_parts.next()?.to_string();

        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                continue;
            }
            if let Some((name, value)) = line.split_once(':') {
                headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
            }
        }

        let content_length = headers
            .get("content-length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let mut body = raw[header_end + 4..].to_vec();
        while body.len() < content_length {
            let read = stream.read(&mut buf).await.ok()?;
            if read == 0 {
                break;
            }
            body.extend_from_slice(&buf[..read]);
        }

        Some(TestRequest {
            method,
            target,
            headers,
        })
    }

    fn find_header_end(bytes: &[u8]) -> Option<usize> {
        bytes.windows(4).position(|window| window == b"\r\n\r\n")
    }

    async fn write_response(stream: &mut TcpStream, response: TestResponse) -> std::io::Result<()> {
        let reason = match response.status {
            204 => "No Content",
            400 => "Bad Request",
            500 => "Internal Server Error",
            _ => "OK",
        };
        let body = response.body;
        let reply = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response.status,
            reason,
            body.len(),
            body
        );
        stream.write_all(reply.as_bytes()).await
    }

    fn ok_json(body: &str) -> TestResponse {
        TestResponse {
            status: 200,
            body: body.to_string(),
        }
    }

    #[tokio::test]
    async fn preflight_failure_returns_api_error() {
        let server = TestServer::spawn(Arc::new(|request| {
            if request.method == "OPTIONS" && request.target.starts_with("/game/queryRole") {
                TestResponse {
                    status: 500,
                    body: "{}".to_string(),
                }
            } else {
                ok_json(r#"{"code":0,"message":"ok","data":{}}"#)
            }
        }))
        .await;

        let client = KuroClient::with_base_url("oauth", &server.base_url).expect("create client");
        let result = client
            .query_role::<serde_json::Value>("12345", "prod_gf_us")
            .await;

        assert!(
            matches!(
                result,
                Err(Error::Client(ClientError::ApiError { code: 500, .. }))
            ),
            "Expected preflight HTTP 500 to map to API error, got: {result:?}"
        );

        let requests = server.requests().await;
        let post_count = requests
            .iter()
            .filter(|request| request.method == "POST")
            .count();
        assert_eq!(post_count, 0, "POST should not run after preflight failure");
    }

    #[tokio::test]
    async fn retry_requested_retries_until_exhausted() {
        let server = TestServer::spawn(Arc::new(|request| {
            if request.method == "OPTIONS" {
                TestResponse {
                    status: 204,
                    body: String::new(),
                }
            } else {
                ok_json(r#"{"code":1005,"message":"retry","data":{}}"#)
            }
        }))
        .await;

        let client = KuroClient::with_base_url("oauth", &server.base_url).expect("create client");
        let result = client
            .query_role::<serde_json::Value>("12345", "prod_gf_us")
            .await;

        assert!(
            matches!(result, Err(Error::RetryRequested)),
            "Expected retry exhaustion to return RetryRequested, got: {result:?}"
        );

        let requests = server.requests().await;
        let options_count = requests.iter().filter(|r| r.method == "OPTIONS").count();
        let post_count = requests.iter().filter(|r| r.method == "POST").count();
        let expected_attempts = usize::try_from(DEFAULT_MAX_RETRIES + 1).expect("u32 to usize");
        assert_eq!(
            options_count, expected_attempts,
            "Expected one preflight per attempt"
        );
        assert_eq!(
            post_count, expected_attempts,
            "Expected one post per attempt"
        );
    }

    #[tokio::test]
    async fn non_retry_error_does_not_retry() {
        let server = TestServer::spawn(Arc::new(|request| {
            if request.method == "OPTIONS" {
                TestResponse {
                    status: 204,
                    body: String::new(),
                }
            } else {
                ok_json(r#"{"code":1234,"message":"hard failure","data":{}}"#)
            }
        }))
        .await;

        let client = KuroClient::with_base_url("oauth", &server.base_url).expect("create client");
        let result = client
            .query_role::<serde_json::Value>("12345", "prod_gf_us")
            .await;

        assert!(
            matches!(
                result,
                Err(Error::Client(ClientError::ApiError { code: 1234, .. }))
            ),
            "Expected non-1005 error to fail immediately, got: {result:?}"
        );

        let requests = server.requests().await;
        let post_count = requests.iter().filter(|r| r.method == "POST").count();
        assert_eq!(post_count, 1, "Non-retryable error should not retry");
    }

    #[tokio::test]
    async fn missing_region_data_returns_nested_parse_error() {
        let server = TestServer::spawn(Arc::new(|request| {
            if request.method == "OPTIONS" {
                TestResponse {
                    status: 204,
                    body: String::new(),
                }
            } else {
                ok_json(r#"{"code":0,"message":"ok","data":{"other":"{\"x\":1}"}}"#)
            }
        }))
        .await;

        let client = KuroClient::with_base_url("oauth", &server.base_url).expect("create client");
        let result = client
            .query_role::<serde_json::Value>("12345", "prod_gf_us")
            .await;

        assert!(
            matches!(
                result,
                Err(Error::NestedDataParseFailed(ref message))
                    if message.contains("No data for region: prod_gf_us")
            ),
            "Expected missing region data parse error, got: {result:?}"
        );
    }

    #[tokio::test]
    async fn request_uses_expected_preflight_headers() {
        let seen_preflight = Arc::new(AtomicUsize::new(0));
        let seen_headers = Arc::new(AtomicUsize::new(0));
        let seen_preflight_clone = Arc::clone(&seen_preflight);
        let seen_headers_clone = Arc::clone(&seen_headers);

        let server = TestServer::spawn(Arc::new(move |request| {
            if request.method == "OPTIONS" {
                seen_preflight_clone.fetch_add(1, Ordering::SeqCst);
                if request
                    .headers
                    .get("access-control-request-method")
                    .is_some_and(|v| v == PREFLIGHT_ACCESS_CONTROL_METHOD)
                    && request
                        .headers
                        .get("access-control-request-headers")
                        .is_some_and(|v| v == PREFLIGHT_ACCESS_CONTROL_HEADERS)
                {
                    seen_headers_clone.fetch_add(1, Ordering::SeqCst);
                }
                TestResponse {
                    status: 204,
                    body: String::new(),
                }
            } else {
                ok_json(
                    r#"{"code":0,"message":"ok","data":{"prod_gf_us":"{\"Base\":{\"Energy\":120,\"MaxEnergy\":240,\"EnergyRecoverTime\":1893456000000}}"}}"#,
                )
            }
        }))
        .await;

        let client = KuroClient::with_base_url("oauth", &server.base_url).expect("create client");
        let result: Result<serde_json::Value> = client.query_role("12345", "prod_gf_us").await;
        assert!(result.is_ok(), "Expected success, got: {result:?}");
        assert_eq!(
            seen_preflight.load(Ordering::SeqCst),
            1,
            "Expected a single preflight request"
        );
        assert_eq!(
            seen_headers.load(Ordering::SeqCst),
            1,
            "Expected preflight headers to match browser semantics"
        );
    }
}
